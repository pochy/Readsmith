use crate::{
    db::now_ts,
    domain::{bookmark::Bookmark, item::Item},
};
use sqlx::SqlitePool;

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Bookmark>> {
    sqlx::query_as::<_, Bookmark>("SELECT id, item_id, link, title, content, feed_title, pub_date, created_at FROM bookmarks ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn is_bookmarked(pool: &SqlitePool, link: &str) -> sqlx::Result<bool> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bookmarks WHERE link = ?1")
        .bind(link)
        .fetch_one(pool)
        .await?;
    Ok(count.0 > 0)
}

pub async fn create_from_item(pool: &SqlitePool, item: &Item) -> sqlx::Result<()> {
    let feed_title: (String,) = sqlx::query_as("SELECT title FROM feeds WHERE id = ?1")
        .bind(item.feed_id)
        .fetch_one(pool)
        .await?;
    sqlx::query(
        r#"
        INSERT INTO bookmarks (item_id, link, title, content, feed_title, pub_date, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(link) DO UPDATE SET
          item_id = excluded.item_id,
          title = excluded.title,
          content = excluded.content,
          feed_title = excluded.feed_title,
          pub_date = excluded.pub_date
        "#,
    )
    .bind(item.id)
    .bind(&item.link)
    .bind(&item.title)
    .bind(&item.content)
    .bind(feed_title.0)
    .bind(item.pub_date)
    .bind(now_ts())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_by_item(pool: &SqlitePool, item: &Item) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM bookmarks WHERE link = ?1")
        .bind(&item.link)
        .execute(pool)
        .await?;
    Ok(())
}
