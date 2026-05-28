use crate::{
    db::{
        feed_repository, group_repository,
        item_repository::{self, ItemFilter},
    },
    domain::{feed::FeedWithGroup, group::Group, item::ItemListEntry},
    error::AppResult,
    state::AppState,
};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, Redirect},
};

#[derive(Template)]
#[template(path = "reader.html")]
pub struct ReaderTemplate {
    pub title: String,
    pub active: String,
    pub groups: Vec<Group>,
    pub feeds: Vec<FeedWithGroup>,
    pub items: Vec<ItemListEntry>,
}

pub async fn home() -> Redirect {
    Redirect::to("/unread")
}

pub async fn unread(State(state): State<AppState>) -> AppResult<Html<String>> {
    reader(state, "Unread", "unread", ItemFilter::Unread).await
}

pub async fn all(State(state): State<AppState>) -> AppResult<Html<String>> {
    reader(state, "All", "all", ItemFilter::All).await
}

pub async fn starred(State(state): State<AppState>) -> AppResult<Html<String>> {
    reader(state, "Starred", "starred", ItemFilter::Starred).await
}

async fn reader(
    state: AppState,
    title: &str,
    active: &str,
    filter: ItemFilter,
) -> AppResult<Html<String>> {
    let groups = group_repository::list(&state.inner.db).await?;
    let feeds = feed_repository::list(&state.inner.db).await?;
    let items = item_repository::list(&state.inner.db, filter, 100).await?;
    Ok(Html(
        ReaderTemplate {
            title: title.to_string(),
            active: active.to_string(),
            groups,
            feeds,
            items,
        }
        .render()?,
    ))
}
