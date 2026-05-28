use crate::{
    db::{bookmark_repository, item_repository},
    error::{AppError, AppResult},
    state::AppState,
};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};

#[derive(Template)]
#[template(path = "partials/article_detail.html")]
struct ArticleDetailTemplate {
    item: crate::domain::item::Item,
    bookmarked: bool,
}

pub async fn detail(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Html<String>> {
    let item = item_repository::get(&state.inner.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let bookmarked = bookmark_repository::is_bookmarked(&state.inner.db, &item.link).await?;
    item_repository::set_unread(&state.inner.db, id, false).await?;
    Ok(Html(ArticleDetailTemplate { item, bookmarked }.render()?))
}

pub async fn mark_read(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Redirect> {
    item_repository::set_unread(&state.inner.db, id, false).await?;
    Ok(Redirect::to("/unread"))
}

pub async fn mark_unread(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Redirect> {
    item_repository::set_unread(&state.inner.db, id, true).await?;
    Ok(Redirect::to("/all"))
}

pub async fn mark_all_read(State(state): State<AppState>) -> AppResult<Redirect> {
    item_repository::mark_all_read(&state.inner.db).await?;
    Ok(Redirect::to("/unread"))
}
