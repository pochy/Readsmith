#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Bookmark {
    pub id: i64,
    pub item_id: Option<i64>,
    pub link: String,
    pub title: String,
    pub content: String,
    pub feed_title: String,
    pub pub_date: i64,
    pub created_at: i64,
}
