use std::{env, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub password: Option<String>,
    pub allow_empty_password: bool,
    pub session_secret: String,
    pub secure_cookies: bool,
    pub pull_interval: Duration,
    pub pull_timeout: Duration,
    pub pull_concurrency: usize,
    pub pull_max_backoff: Duration,
    pub allow_private_feeds: bool,
    pub max_feed_body_bytes: usize,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let password = env::var("APP_PASSWORD").ok().filter(|v| !v.is_empty());
        let allow_empty_password = bool_env("APP_ALLOW_EMPTY_PASSWORD", false);
        if password.is_none() && !allow_empty_password {
            anyhow::bail!("APP_PASSWORD must be set unless APP_ALLOW_EMPTY_PASSWORD=true");
        }
        Ok(Self {
            host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("APP_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:data/readsmith.db".to_string()),
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            password,
            allow_empty_password,
            session_secret: env::var("APP_SESSION_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-me".to_string()),
            secure_cookies: bool_env("APP_SECURE_COOKIES", false),
            pull_interval: Duration::from_secs(u64_env("APP_PULL_INTERVAL_SECONDS", 300)),
            pull_timeout: Duration::from_secs(u64_env("APP_PULL_TIMEOUT_SECONDS", 20)),
            pull_concurrency: usize_env("APP_PULL_CONCURRENCY", 4),
            pull_max_backoff: Duration::from_secs(u64_env("APP_PULL_MAX_BACKOFF_SECONDS", 86_400)),
            allow_private_feeds: bool_env("APP_ALLOW_PRIVATE_FEEDS", false),
            max_feed_body_bytes: usize_env("APP_MAX_FEED_BODY_BYTES", 5 * 1024 * 1024),
        })
    }
}

fn bool_env(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(default)
}

fn u64_env(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn usize_env(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
