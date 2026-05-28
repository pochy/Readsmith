#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FeedFetchState {
    pub feed_id: i64,
    pub etag: String,
    pub last_modified: String,
    pub cache_control: String,
    pub expires_at: i64,
    pub retry_after_until: i64,
    pub last_checked_at: i64,
    pub next_check_at: i64,
    pub last_http_status: i64,
    pub last_success_at: i64,
    pub last_error_at: i64,
    pub last_error: String,
    pub consecutive_failures: i64,
}
