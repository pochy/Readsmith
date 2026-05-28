#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}
