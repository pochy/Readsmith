use crate::{
    db::{bookmark_repository, item_repository},
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    response::Redirect,
};

pub async fn bookmark(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Redirect> {
    let item = item_repository::get(&state.inner.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    bookmark_repository::create_from_item(&state.inner.db, &item).await?;
    Ok(Redirect::to("/starred"))
}

pub async fn unbookmark(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Redirect> {
    let item = item_repository::get(&state.inner.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    bookmark_repository::delete_by_item(&state.inner.db, &item).await?;
    Ok(Redirect::to("/starred"))
}
