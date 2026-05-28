pub mod auth;
pub mod bookmarks;
pub mod feeds;
pub mod groups;
pub mod items;
pub mod opml;
pub mod pages;
pub mod search;
pub mod static_files;

use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(static_files::healthz))
        .route("/login", get(auth::login_page).post(auth::login))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(pages::home))
        .route("/unread", get(pages::unread))
        .route("/all", get(pages::all))
        .route("/starred", get(pages::starred))
        .route("/logout", post(auth::logout))
        .route("/feeds", get(feeds::index).post(feeds::create))
        .route("/feeds/new", get(feeds::new_form))
        .route("/feeds/{id}/edit", get(feeds::edit_form))
        .route("/feeds/{id}", post(feeds::update))
        .route("/feeds/{id}/delete", post(feeds::delete))
        .route("/feeds/{id}/refresh", post(feeds::refresh))
        .route("/groups", get(groups::index).post(groups::create))
        .route("/groups/{id}", post(groups::update))
        .route("/groups/{id}/delete", post(groups::delete))
        .route("/items/{id}", get(items::detail))
        .route("/items/{id}/read", post(items::mark_read))
        .route("/items/{id}/unread", post(items::mark_unread))
        .route("/items/{id}/bookmark", post(bookmarks::bookmark))
        .route("/items/{id}/unbookmark", post(bookmarks::unbookmark))
        .route("/items/mark-all-read", post(items::mark_all_read))
        .route("/search", get(search::search))
        .route("/opml/export", get(opml::export))
        .route("/opml/import", post(opml::import))
}
