#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Feed {
    pub id: i64,
    pub group_id: i64,
    pub title: String,
    pub feed_url: String,
    pub site_url: String,
    pub description: String,
    pub suspended: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FeedWithGroup {
    pub id: i64,
    pub group_id: i64,
    pub group_name: String,
    pub title: String,
    pub feed_url: String,
    pub site_url: String,
    pub description: String,
    pub suspended: i64,
    pub unread_count: i64,
}
