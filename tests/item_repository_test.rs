use readsmith::{
    db::{item_repository, migration},
    services::feed_parser::ParsedItem,
};
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn upsert_many_preserves_unread_on_existing_item() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    migration::run(&pool).await.unwrap();
    sqlx::query("INSERT INTO feeds (id, group_id, title, feed_url, created_at, updated_at) VALUES (1, 1, 'Feed', 'https://example.com/feed.xml', 0, 0)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO feed_fetch_state (feed_id) VALUES (1)")
        .execute(&pool)
        .await
        .unwrap();
    let item = ParsedItem {
        guid: "g1".to_string(),
        title: "First".to_string(),
        link: "https://example.com/1".to_string(),
        content: "Content".to_string(),
        author: String::new(),
        pub_date: 1,
    };
    item_repository::upsert_many(&pool, 1, &[item.clone()])
        .await
        .unwrap();
    item_repository::set_unread(&pool, 1, false).await.unwrap();
    let mut changed = item;
    changed.title = "First updated".to_string();
    item_repository::upsert_many(&pool, 1, &[changed])
        .await
        .unwrap();
    let stored = item_repository::get(&pool, 1).await.unwrap().unwrap();
    assert_eq!(stored.title, "First updated");
    assert_eq!(stored.unread, 0);
}
