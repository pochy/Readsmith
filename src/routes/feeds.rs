use crate::{
    db::{feed_repository, group_repository},
    domain::{
        feed::{Feed, FeedWithGroup},
        group::Group,
    },
    error::{AppError, AppResult},
    services::{feed_discovery, feed_fetcher, url_guard::UrlGuard},
    state::AppState,
};
use askama::Template;
use axum::{
    extract::{Form, Path, State},
    response::{Html, Redirect},
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "feeds.html")]
struct FeedsTemplate {
    active: String,
    groups: Vec<Group>,
    feeds: Vec<FeedWithGroup>,
}

#[derive(Template)]
#[template(path = "partials/feed_form.html")]
struct FeedFormTemplate {
    groups: Vec<Group>,
    feed: Option<Feed>,
}

#[derive(Deserialize)]
pub struct FeedForm {
    group_id: i64,
    title: String,
    feed_url: String,
    site_url: Option<String>,
    description: Option<String>,
    suspended: Option<String>,
}

pub async fn index(State(state): State<AppState>) -> AppResult<Html<String>> {
    Ok(Html(
        FeedsTemplate {
            active: "feeds".to_string(),
            groups: group_repository::list(&state.inner.db).await?,
            feeds: feed_repository::list(&state.inner.db).await?,
        }
        .render()?,
    ))
}

pub async fn new_form(State(state): State<AppState>) -> AppResult<Html<String>> {
    Ok(Html(
        FeedFormTemplate {
            groups: group_repository::list(&state.inner.db).await?,
            feed: None,
        }
        .render()?,
    ))
}

pub async fn edit_form(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Html<String>> {
    let feed = feed_repository::get(&state.inner.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Html(
        FeedFormTemplate {
            groups: group_repository::list(&state.inner.db).await?,
            feed: Some(feed),
        }
        .render()?,
    ))
}

pub async fn create(
    State(state): State<AppState>,
    Form(mut form): Form<FeedForm>,
) -> AppResult<Redirect> {
    normalize_feed_form(&state, &mut form).await?;
    feed_repository::create(
        &state.inner.db,
        form.group_id,
        &form.title,
        &form.feed_url,
        form.site_url.as_deref().unwrap_or(""),
        form.description.as_deref().unwrap_or(""),
    )
    .await?;
    Ok(Redirect::to("/feeds"))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(mut form): Form<FeedForm>,
) -> AppResult<Redirect> {
    normalize_feed_form(&state, &mut form).await?;
    feed_repository::update(
        &state.inner.db,
        id,
        form.group_id,
        &form.title,
        &form.feed_url,
        form.site_url.as_deref().unwrap_or(""),
        form.description.as_deref().unwrap_or(""),
        form.suspended.is_some(),
    )
    .await?;
    Ok(Redirect::to("/feeds"))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Redirect> {
    feed_repository::delete(&state.inner.db, id).await?;
    Ok(Redirect::to("/feeds"))
}

pub async fn refresh(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Redirect> {
    let feed = feed_repository::get(&state.inner.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let _ = feed_fetcher::refresh_feed(&state, &feed).await;
    Ok(Redirect::to("/feeds"))
}

async fn normalize_feed_form(state: &AppState, form: &mut FeedForm) -> AppResult<()> {
    if form.feed_url.trim().is_empty() {
        return Err(AppError::BadRequest("feed URL is required".to_string()));
    }
    let discovered = feed_discovery::discover(state, &form.feed_url)
        .await
        .unwrap_or_else(|_| form.feed_url.trim().to_string());
    UrlGuard::new(state.inner.config.allow_private_feeds)
        .validate_fetch_url(&discovered)
        .await?;
    form.feed_url = discovered;
    if form.title.trim().is_empty() {
        form.title = form.feed_url.clone();
    }
    Ok(())
}
