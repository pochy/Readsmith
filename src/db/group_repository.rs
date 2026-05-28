use crate::{db::now_ts, domain::group::Group};
use sqlx::SqlitePool;

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Group>> {
    sqlx::query_as::<_, Group>("SELECT id, name, created_at, updated_at FROM groups ORDER BY name")
        .fetch_all(pool)
        .await
}

pub async fn create(pool: &SqlitePool, name: &str) -> sqlx::Result<i64> {
    let now = now_ts();
    let res = sqlx::query("INSERT INTO groups (name, created_at, updated_at) VALUES (?1, ?2, ?3)")
        .bind(name.trim())
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
    Ok(res.last_insert_rowid())
}

pub async fn update(pool: &SqlitePool, id: i64, name: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE groups SET name = ?1, updated_at = ?2 WHERE id = ?3 AND id != 1")
        .bind(name.trim())
        .bind(now_ts())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE feeds SET group_id = 1, updated_at = ?1 WHERE group_id = ?2")
        .bind(now_ts())
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM groups WHERE id = ?1 AND id != 1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}
