use crate::{
    db::{feed_repository, item_repository},
    domain::feed::Feed,
    error::{AppError, AppResult},
    services::{
        feed_discovery::is_feed_content_type, feed_parser, pull_policy, url_guard::UrlGuard,
    },
    state::AppState,
};
use axum::http::StatusCode;
use reqwest::header::{
    CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH,
    LAST_MODIFIED, LOCATION,
};
use time::OffsetDateTime;

pub async fn refresh_feed(state: &AppState, feed: &Feed) -> AppResult<usize> {
    let guard = UrlGuard::new(state.inner.config.allow_private_feeds);
    let mut url = guard.validate_fetch_url(&feed.feed_url).await?;
    let fetch_state = feed_repository::fetch_state(&state.inner.db, feed.id).await?;
    let mut redirects = 0;
    loop {
        let mut req = state.inner.http.get(url.clone());
        if let Some(s) = &fetch_state {
            if !s.etag.is_empty() {
                req = req.header(IF_NONE_MATCH, &s.etag);
            }
            if !s.last_modified.is_empty() {
                req = req.header(IF_MODIFIED_SINCE, &s.last_modified);
            }
        }
        let response = req
            .send()
            .await
            .map_err(|e| AppError::Feed(e.to_string()))?;
        if response.status().is_redirection() {
            redirects += 1;
            if redirects > 5 {
                return record_error(
                    state,
                    feed.id,
                    response.status().as_u16(),
                    "too many redirects",
                )
                .await;
            }
            let location = response
                .headers()
                .get(LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| AppError::Feed("redirect without Location".to_string()))?;
            url = url
                .join(location)
                .map_err(|e| AppError::Feed(format!("invalid redirect: {e}")))?;
            guard.validate_fetch_url(url.as_str()).await?;
            continue;
        }
        if response.status() == StatusCode::NOT_MODIFIED {
            let next = pull_policy::next_after_success(
                OffsetDateTime::now_utc(),
                fetch_state.as_ref().map(|s| s.cache_control.as_str()),
            )
            .unix_timestamp();
            feed_repository::record_not_modified(
                &state.inner.db,
                feed.id,
                response.status().as_u16(),
                next,
            )
            .await?;
            return Ok(0);
        }
        if !response.status().is_success() {
            return record_error(
                state,
                feed.id,
                response.status().as_u16(),
                &format!("HTTP {}", response.status()),
            )
            .await;
        }
        let headers = response.headers().clone();
        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !is_feed_content_type(content_type) {
            return record_error(
                state,
                feed.id,
                response.status().as_u16(),
                "response is not a feed content type",
            )
            .await;
        }
        if let Some(len) = headers
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
        {
            if len > state.inner.config.max_feed_body_bytes {
                return record_error(
                    state,
                    feed.id,
                    response.status().as_u16(),
                    "feed body is too large",
                )
                .await;
            }
        }
        let status = response.status().as_u16();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Feed(e.to_string()))?;
        if bytes.len() > state.inner.config.max_feed_body_bytes {
            return record_error(state, feed.id, status, "feed body is too large").await;
        }
        let parsed = feed_parser::parse_feed(&bytes).map_err(|e| AppError::Feed(e.to_string()))?;
        let inserted =
            item_repository::upsert_many(&state.inner.db, feed.id, &parsed.items).await?;
        let etag = headers
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let last_modified = headers
            .get(LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let cache_control = headers
            .get(CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let next = pull_policy::next_after_success(OffsetDateTime::now_utc(), Some(cache_control))
            .unix_timestamp();
        feed_repository::record_success(
            &state.inner.db,
            feed.id,
            etag,
            last_modified,
            cache_control,
            status,
            next,
        )
        .await?;
        return Ok(inserted);
    }
}

async fn record_error(
    state: &AppState,
    feed_id: i64,
    status: u16,
    error: &str,
) -> AppResult<usize> {
    let fetch_state = feed_repository::fetch_state(&state.inner.db, feed_id).await?;
    let failures = fetch_state
        .map(|s| s.consecutive_failures as u32 + 1)
        .unwrap_or(1);
    let next = pull_policy::next_after_error(
        OffsetDateTime::now_utc(),
        failures,
        state.inner.config.pull_max_backoff.as_secs(),
    )
    .unix_timestamp();
    feed_repository::record_error(&state.inner.db, feed_id, status, error, next).await?;
    Err(AppError::Feed(error.to_string()))
}
