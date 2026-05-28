use crate::{
    config::Config,
    routes,
    state::{self, AppState},
};
use axum::{Router, middleware};
use sqlx::SqlitePool;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn build_state(config: Config, pool: SqlitePool) -> anyhow::Result<AppState> {
    let http = state::http_client(&config)?;
    Ok(AppState::new(config, pool, http))
}

pub fn router(state: AppState) -> Router {
    let protected = routes::protected_routes().layer(middleware::from_fn_with_state(
        state.clone(),
        routes::auth::require_auth,
    ));

    Router::new()
        .merge(routes::public_routes())
        .merge(protected)
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
