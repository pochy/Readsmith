use crate::domain::item::ItemListEntry;
use sqlx::SqlitePool;

pub async fn search_items(
    pool: &SqlitePool,
    q: &str,
    limit: i64,
) -> sqlx::Result<Vec<ItemListEntry>> {
    sqlx::query_as::<_, ItemListEntry>(
        r#"
        SELECT i.id, i.feed_id, f.title AS feed_title, i.title, i.link, i.author, i.pub_date, i.unread,
               CASE WHEN b.id IS NULL THEN 0 ELSE 1 END AS bookmarked
        FROM items_fts
        JOIN items i ON i.id = items_fts.rowid
        JOIN feeds f ON f.id = i.feed_id
        LEFT JOIN bookmarks b ON b.link = i.link
        WHERE items_fts MATCH ?1
        ORDER BY bm25(items_fts)
        LIMIT ?2
        "#,
    )
    .bind(q)
    .bind(limit)
    .fetch_all(pool)
    .await
}
