use crate::{
    db::{feed_repository, group_repository},
    error::AppResult,
    services,
    state::AppState,
};
use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Redirect},
};
use std::collections::HashSet;

pub async fn export(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let groups = group_repository::list(&state.inner.db).await?;
    let feeds = feed_repository::list(&state.inner.db).await?;
    let body = services::opml::export(&groups, &feeds);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/xml; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"readsmith.opml\""),
    );
    Ok((headers, body))
}

pub async fn import(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Redirect> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| crate::error::AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("opml") {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| crate::error::AppError::BadRequest(e.to_string()))?;
            let feeds = services::opml::parse_import(&bytes)?;
            let existing: HashSet<String> = feed_repository::list(&state.inner.db)
                .await?
                .into_iter()
                .map(|f| f.feed_url)
                .collect();
            for feed in feeds {
                if existing.contains(&feed.xml_url) {
                    continue;
                }
                let groups = group_repository::list(&state.inner.db).await?;
                let group_id = groups
                    .iter()
                    .find(|g| g.name == feed.group_name)
                    .map(|g| g.id)
                    .unwrap_or(group_repository::create(&state.inner.db, &feed.group_name).await?);
                let _ = feed_repository::create(
                    &state.inner.db,
                    group_id,
                    &feed.title,
                    &feed.xml_url,
                    &feed.html_url,
                    "",
                )
                .await;
            }
            break;
        }
    }
    Ok(Redirect::to("/feeds"))
}
