#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub link: String,
    pub content: String,
    pub author: String,
    pub pub_date: i64,
    pub unread: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemListEntry {
    pub id: i64,
    pub feed_id: i64,
    pub feed_title: String,
    pub title: String,
    pub link: String,
    pub author: String,
    pub pub_date: i64,
    pub unread: i64,
    pub bookmarked: i64,
}
