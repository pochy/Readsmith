use crate::{
    db::{feed_repository, group_repository, search_repository},
    domain::{feed::FeedWithGroup, group::Group, item::ItemListEntry},
    error::AppResult,
    state::AppState,
};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    active: String,
    q: String,
    groups: Vec<Group>,
    feeds: Vec<FeedWithGroup>,
    items: Vec<ItemListEntry>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Html<String>> {
    let q = query.q.unwrap_or_default();
    let items = if q.trim().is_empty() {
        Vec::new()
    } else {
        search_repository::search_items(&state.inner.db, &q, 100).await?
    };
    Ok(Html(
        SearchTemplate {
            active: "search".to_string(),
            q,
            groups: group_repository::list(&state.inner.db).await?,
            feeds: feed_repository::list(&state.inner.db).await?,
            items,
        }
        .render()?,
    ))
}
