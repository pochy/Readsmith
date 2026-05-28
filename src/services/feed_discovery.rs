use crate::{
    error::{AppError, AppResult},
    services::url_guard::UrlGuard,
    state::AppState,
};
use scraper::{Html, Selector};
use url::Url;

pub async fn discover(state: &AppState, raw_url: &str) -> AppResult<String> {
    let guard = UrlGuard::new(state.inner.config.allow_private_feeds);
    let url = guard.validate_fetch_url(raw_url).await?;
    let response = state
        .inner
        .http
        .get(url.clone())
        .send()
        .await
        .map_err(|e| AppError::Feed(e.to_string()))?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if is_feed_content_type(content_type) {
        return Ok(url.to_string());
    }
    let base = response.url().clone();
    let body = response
        .text()
        .await
        .map_err(|e| AppError::Feed(e.to_string()))?;
    let discovered = {
        let doc = Html::parse_document(&body);
        let selector = Selector::parse(r#"link[rel~="alternate"]"#)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        doc.select(&selector).find_map(|node| {
            let mime = node.value().attr("type").unwrap_or("");
            if is_feed_content_type(mime) {
                node.value()
                    .attr("href")
                    .and_then(|href| resolve_url(&base, href).ok())
            } else {
                None
            }
        })
    };
    if let Some(discovered) = discovered {
        guard.validate_fetch_url(discovered.as_str()).await?;
        return Ok(discovered.to_string());
    }
    Err(AppError::BadRequest(
        "feed URL could not be discovered".to_string(),
    ))
}

fn resolve_url(base: &Url, href: &str) -> AppResult<Url> {
    base.join(href)
        .map_err(|e| AppError::BadRequest(format!("invalid discovered feed URL: {e}")))
}

pub fn is_feed_content_type(content_type: &str) -> bool {
    let content_type = content_type.to_ascii_lowercase();
    content_type.contains("rss")
        || content_type.contains("atom")
        || content_type.contains("feed")
        || content_type.contains("xml")
        || content_type.contains("json")
}
