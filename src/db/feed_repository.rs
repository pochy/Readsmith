use crate::{
    db::now_ts,
    domain::{
        feed::{Feed, FeedWithGroup},
        feed_fetch_state::FeedFetchState,
    },
};
use sqlx::SqlitePool;

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<FeedWithGroup>> {
    sqlx::query_as::<_, FeedWithGroup>(
        r#"
        SELECT f.id, f.group_id, g.name AS group_name, f.title, f.feed_url, f.site_url,
               f.description, f.suspended,
               COALESCE(SUM(CASE WHEN i.unread = 1 THEN 1 ELSE 0 END), 0) AS unread_count
        FROM feeds f
        JOIN groups g ON g.id = f.group_id
        LEFT JOIN items i ON i.feed_id = f.id
        GROUP BY f.id
        ORDER BY g.name, f.title
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Feed>> {
    sqlx::query_as::<_, Feed>("SELECT id, group_id, title, feed_url, site_url, description, suspended, created_at, updated_at FROM feeds WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(
    pool: &SqlitePool,
    group_id: i64,
    title: &str,
    feed_url: &str,
    site_url: &str,
    description: &str,
) -> sqlx::Result<i64> {
    let now = now_ts();
    let mut tx = pool.begin().await?;
    let res = sqlx::query(
        "INSERT INTO feeds (group_id, title, feed_url, site_url, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(group_id)
    .bind(title.trim())
    .bind(feed_url.trim())
    .bind(site_url.trim())
    .bind(description.trim())
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    let id = res.last_insert_rowid();
    sqlx::query("INSERT INTO feed_fetch_state (feed_id, next_check_at) VALUES (?1, 0)")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    group_id: i64,
    title: &str,
    feed_url: &str,
    site_url: &str,
    description: &str,
    suspended: bool,
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE feeds SET group_id = ?1, title = ?2, feed_url = ?3, site_url = ?4, description = ?5, suspended = ?6, updated_at = ?7 WHERE id = ?8",
    )
    .bind(group_id)
    .bind(title.trim())
    .bind(feed_url.trim())
    .bind(site_url.trim())
    .bind(description.trim())
    .bind(if suspended { 1 } else { 0 })
    .bind(now_ts())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM feeds WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn due_for_refresh(pool: &SqlitePool, now: i64, limit: i64) -> sqlx::Result<Vec<Feed>> {
    sqlx::query_as::<_, Feed>(
        r#"
        SELECT f.id, f.group_id, f.title, f.feed_url, f.site_url, f.description, f.suspended, f.created_at, f.updated_at
        FROM feeds f
        JOIN feed_fetch_state s ON s.feed_id = f.id
        WHERE f.suspended = 0 AND s.next_check_at <= ?1
        ORDER BY s.next_check_at ASC
        LIMIT ?2
        "#,
    )
    .bind(now)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn fetch_state(pool: &SqlitePool, feed_id: i64) -> sqlx::Result<Option<FeedFetchState>> {
    sqlx::query_as::<_, FeedFetchState>("SELECT * FROM feed_fetch_state WHERE feed_id = ?1")
        .bind(feed_id)
        .fetch_optional(pool)
        .await
}

pub async fn record_success(
    pool: &SqlitePool,
    feed_id: i64,
    etag: &str,
    last_modified: &str,
    cache_control: &str,
    status: u16,
    next_check_at: i64,
) -> sqlx::Result<()> {
    let now = now_ts();
    sqlx::query(
        r#"
        UPDATE feed_fetch_state
        SET etag = ?1, last_modified = ?2, cache_control = ?3, last_checked_at = ?4,
            next_check_at = ?5, last_http_status = ?6, last_success_at = ?4,
            last_error_at = 0, last_error = '', consecutive_failures = 0
        WHERE feed_id = ?7
        "#,
    )
    .bind(etag)
    .bind(last_modified)
    .bind(cache_control)
    .bind(now)
    .bind(next_check_at)
    .bind(status as i64)
    .bind(feed_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn record_not_modified(
    pool: &SqlitePool,
    feed_id: i64,
    status: u16,
    next_check_at: i64,
) -> sqlx::Result<()> {
    let now = now_ts();
    sqlx::query(
        "UPDATE feed_fetch_state SET last_checked_at = ?1, next_check_at = ?2, last_http_status = ?3, last_success_at = ?1, last_error = '', consecutive_failures = 0 WHERE feed_id = ?4",
    )
    .bind(now)
    .bind(next_check_at)
    .bind(status as i64)
    .bind(feed_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn record_error(
    pool: &SqlitePool,
    feed_id: i64,
    status: u16,
    error: &str,
    next_check_at: i64,
) -> sqlx::Result<()> {
    let now = now_ts();
    sqlx::query(
        r#"
        UPDATE feed_fetch_state
        SET last_checked_at = ?1, next_check_at = ?2, last_http_status = ?3,
            last_error_at = ?1, last_error = ?4, consecutive_failures = consecutive_failures + 1
        WHERE feed_id = ?5
        "#,
    )
    .bind(now)
    .bind(next_check_at)
    .bind(status as i64)
    .bind(error.chars().take(500).collect::<String>())
    .bind(feed_id)
    .execute(pool)
    .await?;
    Ok(())
}
