use crate::{
    domain::{feed::FeedWithGroup, group::Group},
    error::{AppError, AppResult},
};
use quick_xml::{Reader, events::Event};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpmlFeed {
    pub group_name: String,
    pub title: String,
    pub xml_url: String,
    pub html_url: String,
}

pub fn parse_import(bytes: &[u8]) -> AppResult<Vec<OpmlFeed>> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut current_group = "Default".to_string();
    let mut feeds = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) if e.name().as_ref() == b"outline" => {
                let mut text = String::new();
                let mut title = String::new();
                let mut xml_url = String::new();
                let mut html_url = String::new();
                for attr in e.attributes().flatten() {
                    let key = attr.key.as_ref();
                    let value = attr
                        .unescape_value()
                        .map(|v| v.to_string())
                        .unwrap_or_default();
                    match key {
                        b"text" => text = value,
                        b"title" => title = value,
                        b"xmlUrl" => xml_url = value,
                        b"htmlUrl" => html_url = value,
                        _ => {}
                    }
                }
                if xml_url.is_empty() {
                    current_group = if !title.is_empty() {
                        title
                    } else if !text.is_empty() {
                        text
                    } else {
                        current_group
                    };
                } else {
                    feeds.push(OpmlFeed {
                        group_name: current_group.clone(),
                        title: if !title.is_empty() { title } else { text },
                        xml_url,
                        html_url,
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(AppError::BadRequest(format!("invalid OPML: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(feeds)
}

pub fn export(groups: &[Group], feeds: &[FeedWithGroup]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <head><title>Readsmith subscriptions</title></head>
  <body>
"#,
    );
    for group in groups {
        xml.push_str(&format!(
            "    <outline text=\"{}\" title=\"{}\">\n",
            esc(&group.name),
            esc(&group.name)
        ));
        for feed in feeds.iter().filter(|f| f.group_id == group.id) {
            xml.push_str(&format!(
                "      <outline type=\"rss\" text=\"{}\" title=\"{}\" xmlUrl=\"{}\" htmlUrl=\"{}\" />\n",
                esc(&feed.title),
                esc(&feed.title),
                esc(&feed.feed_url),
                esc(&feed.site_url)
            ));
        }
        xml.push_str("    </outline>\n");
    }
    xml.push_str("  </body>\n</opml>\n");
    xml
}

fn esc(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
