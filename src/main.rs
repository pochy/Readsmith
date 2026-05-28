use readsmith::{
    app,
    config::{Config, sqlite_database_parent},
    db::migration,
    services::pull_worker,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "readsmith=info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env()?;
    if let Some(parent) = sqlite_database_parent(&config.database_url) {
        tokio::fs::create_dir_all(parent).await?;
    }
    let connect_options =
        SqliteConnectOptions::from_str(&config.database_url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect_with(connect_options)
        .await?;
    migration::run(&pool).await?;

    let state = app::build_state(config.clone(), pool)?;
    let worker_state = state.clone();
    tokio::spawn(async move {
        pull_worker::run(worker_state).await;
    });

    let router = app::router(state);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "readsmith listening");
    axum::serve(listener, router).await?;
    Ok(())
}
