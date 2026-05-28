use crate::config::Config;
use reqwest::{Client, redirect::Policy};
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub config: Config,
    pub db: SqlitePool,
    pub http: Client,
}

impl AppState {
    pub fn new(config: Config, db: SqlitePool, http: Client) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config, db, http }),
        }
    }
}

pub fn http_client(config: &Config) -> anyhow::Result<Client> {
    Ok(Client::builder()
        .timeout(config.pull_timeout)
        .redirect(Policy::none())
        .user_agent("Readsmith/0.1")
        .build()?)
}
