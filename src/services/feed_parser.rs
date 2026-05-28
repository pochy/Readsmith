use crate::services::sanitizer::sanitize_html;
use feed_rs::model::{Entry, Feed};
use sha2::{Digest, Sha256};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct ParsedFeed {
    pub title: String,
    pub site_url: String,
    pub description: String,
    pub items: Vec<ParsedItem>,
}

#[derive(Debug, Clone)]
pub struct ParsedItem {
    pub guid: String,
    pub title: String,
    pub link: String,
    pub content: String,
    pub author: String,
    pub pub_date: i64,
}

pub fn parse_feed(bytes: &[u8]) -> anyhow::Result<ParsedFeed> {
    let feed = feed_rs::parser::parse(Cursor::new(bytes))?;
    Ok(map_feed(feed))
}

fn map_feed(feed: Feed) -> ParsedFeed {
    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled feed".to_string());
    let site_url = feed
        .links
        .first()
        .map(|l| l.href.clone())
        .unwrap_or_default();
    let description = feed
        .description
        .map(|d| sanitize_html(&d.content))
        .unwrap_or_default();
    let items = feed.entries.iter().map(map_entry).collect();
    ParsedFeed {
        title,
        site_url,
        description,
        items,
    }
}

fn map_entry(entry: &Entry) -> ParsedItem {
    let title = entry
        .title
        .as_ref()
        .map(|t| t.content.clone())
        .unwrap_or_else(|| "Untitled".to_string());
    let link = entry
        .links
        .first()
        .map(|l| l.href.clone())
        .unwrap_or_default();
    let pub_date = entry
        .published
        .or(entry.updated)
        .map(|d| d.timestamp())
        .unwrap_or(0);
    let content = entry
        .content
        .as_ref()
        .and_then(|c| c.body.clone())
        .or_else(|| entry.summary.as_ref().map(|s| s.content.clone()))
        .unwrap_or_else(|| title.clone());
    let author = entry
        .authors
        .first()
        .map(|p| p.name.clone())
        .unwrap_or_default();
    let guid = choose_guid(entry, &link, &title, pub_date);
    ParsedItem {
        guid,
        title,
        link,
        content: sanitize_html(&content),
        author,
        pub_date,
    }
}

pub fn choose_guid(entry: &Entry, link: &str, title: &str, pub_date: i64) -> String {
    if !entry.id.trim().is_empty() && !looks_generated_feed_rs_id(&entry.id) {
        return entry.id.clone();
    }
    if !link.trim().is_empty() {
        return link.to_string();
    }
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(pub_date.to_string().as_bytes());
    format!("{:x}", hasher.finalize())
}

fn looks_generated_feed_rs_id(id: &str) -> bool {
    (id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()))
        || (id.len() == 36
            && id.bytes().enumerate().all(|(idx, b)| {
                matches!(idx, 8 | 13 | 18 | 23) && b == b'-'
                    || !matches!(idx, 8 | 13 | 18 | 23) && b.is_ascii_hexdigit()
            }))
}
