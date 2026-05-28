pub mod bookmark_repository;
pub mod feed_repository;
pub mod group_repository;
pub mod item_repository;
pub mod migration;
pub mod search_repository;

pub fn now_ts() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}
