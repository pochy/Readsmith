use crate::{
    db::now_ts,
    domain::item::{Item, ItemListEntry},
    services::feed_parser::ParsedItem,
};
use sqlx::{Sqlite, SqlitePool, Transaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemFilter {
    Unread,
    All,
    Starred,
}

pub async fn list(
    pool: &SqlitePool,
    filter: ItemFilter,
    limit: i64,
) -> sqlx::Result<Vec<ItemListEntry>> {
    let sql = match filter {
        ItemFilter::Unread => BASE_LIST_SQL_UNREAD,
        ItemFilter::All => BASE_LIST_SQL_ALL,
        ItemFilter::Starred => BASE_LIST_SQL_STARRED,
    };
    sqlx::query_as::<_, ItemListEntry>(sql)
        .bind(limit)
        .fetch_all(pool)
        .await
}

const BASE_LIST_SQL_UNREAD: &str = r#"
SELECT i.id, i.feed_id, f.title AS feed_title, i.title, i.link, i.author, i.pub_date, i.unread,
       CASE WHEN b.id IS NULL THEN 0 ELSE 1 END AS bookmarked
FROM items i
JOIN feeds f ON f.id = i.feed_id
LEFT JOIN bookmarks b ON b.link = i.link
WHERE i.unread = 1 ORDER BY i.pub_date DESC, i.id DESC LIMIT ?1
"#;
const BASE_LIST_SQL_ALL: &str = r#"
SELECT i.id, i.feed_id, f.title AS feed_title, i.title, i.link, i.author, i.pub_date, i.unread,
       CASE WHEN b.id IS NULL THEN 0 ELSE 1 END AS bookmarked
FROM items i
JOIN feeds f ON f.id = i.feed_id
LEFT JOIN bookmarks b ON b.link = i.link
ORDER BY i.pub_date DESC, i.id DESC LIMIT ?1
"#;
const BASE_LIST_SQL_STARRED: &str = r#"
SELECT i.id, i.feed_id, f.title AS feed_title, i.title, i.link, i.author, i.pub_date, i.unread,
       CASE WHEN b.id IS NULL THEN 0 ELSE 1 END AS bookmarked
FROM items i
JOIN feeds f ON f.id = i.feed_id
LEFT JOIN bookmarks b ON b.link = i.link
WHERE b.id IS NOT NULL ORDER BY b.created_at DESC LIMIT ?1
"#;

pub async fn get(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Item>> {
    sqlx::query_as::<_, Item>("SELECT id, feed_id, guid, title, link, content, author, pub_date, unread, created_at, updated_at FROM items WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn set_unread(pool: &SqlitePool, id: i64, unread: bool) -> sqlx::Result<()> {
    sqlx::query("UPDATE items SET unread = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(if unread { 1 } else { 0 })
        .bind(now_ts())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_all_read(pool: &SqlitePool) -> sqlx::Result<u64> {
    let res = sqlx::query("UPDATE items SET unread = 0, updated_at = ?1 WHERE unread = 1")
        .bind(now_ts())
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

pub async fn upsert_many(
    pool: &SqlitePool,
    feed_id: i64,
    items: &[ParsedItem],
) -> sqlx::Result<usize> {
    let mut tx = pool.begin().await?;
    let inserted = upsert_many_tx(&mut tx, feed_id, items).await?;
    tx.commit().await?;
    Ok(inserted)
}

pub async fn upsert_many_tx(
    tx: &mut Transaction<'_, Sqlite>,
    feed_id: i64,
    items: &[ParsedItem],
) -> sqlx::Result<usize> {
    let mut inserted = 0;
    for item in items {
        let now = now_ts();
        let res = sqlx::query(
            r#"
            INSERT INTO items (feed_id, guid, title, link, content, author, pub_date, unread, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)
            ON CONFLICT(feed_id, guid) DO UPDATE SET
              title = excluded.title,
              link = excluded.link,
              content = excluded.content,
              author = excluded.author,
              pub_date = excluded.pub_date,
              updated_at = excluded.updated_at
            "#,
        )
        .bind(feed_id)
        .bind(&item.guid)
        .bind(&item.title)
        .bind(&item.link)
        .bind(&item.content)
        .bind(&item.author)
        .bind(item.pub_date)
        .bind(now)
        .execute(&mut **tx)
        .await?;
        if res.rows_affected() == 1 {
            inserted += 1;
        }
    }
    Ok(inserted)
}
