use readsmith::config::{Config, sqlite_database_parent};
use std::sync::{Mutex, OnceLock};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn clear_app_env() {
    for key in [
        "APP_PASSWORD",
        "APP_ALLOW_EMPTY_PASSWORD",
        "APP_HOST",
        "APP_PORT",
        "DATABASE_URL",
        "DATABASE_MAX_CONNECTIONS",
        "APP_SESSION_SECRET",
        "APP_SECURE_COOKIES",
        "APP_PULL_INTERVAL_SECONDS",
        "APP_PULL_TIMEOUT_SECONDS",
        "APP_PULL_CONCURRENCY",
        "APP_PULL_MAX_BACKOFF_SECONDS",
        "APP_ALLOW_PRIVATE_FEEDS",
        "APP_MAX_FEED_BODY_BYTES",
    ] {
        unsafe {
            std::env::remove_var(key);
        }
    }
}

#[test]
fn config_loads_env_file_from_current_directory() {
    let _guard = env_lock();
    let original_dir = std::env::current_dir().unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::write(
        temp_dir.path().join(".env"),
        "APP_PASSWORD=from-dotenv\nAPP_PORT=9091\n",
    )
    .unwrap();

    clear_app_env();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = Config::from_env();

    std::env::set_current_dir(original_dir).unwrap();
    clear_app_env();

    let config = config.unwrap();
    assert_eq!(config.password.as_deref(), Some("from-dotenv"));
    assert_eq!(config.port, 9091);
}

#[test]
fn sqlite_database_parent_returns_file_parent_directory() {
    assert_eq!(
        sqlite_database_parent("sqlite:data/readsmith.db").as_deref(),
        Some(std::path::Path::new("data"))
    );
    assert_eq!(
        sqlite_database_parent("sqlite:/data/readsmith.db").as_deref(),
        Some(std::path::Path::new("/data"))
    );
}

#[test]
fn sqlite_database_parent_ignores_memory_and_non_sqlite_urls() {
    assert_eq!(sqlite_database_parent("sqlite::memory:"), None);
    assert_eq!(sqlite_database_parent("postgres://localhost/readsmith"), None);
}
