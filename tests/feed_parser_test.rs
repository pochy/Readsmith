use readsmith::services::feed_parser::parse_feed;

#[test]
fn parses_rss_and_sanitizes_content() {
    let xml = br#"<?xml version="1.0"?>
    <rss version="2.0"><channel>
      <title>Example</title>
      <link>https://example.com</link>
      <description>Feed</description>
      <item>
        <title>Hello</title>
        <link>https://example.com/hello</link>
        <description><![CDATA[<p onclick="bad()">Hi<script>alert(1)</script></p>]]></description>
        <pubDate>Thu, 28 May 2026 00:00:00 GMT</pubDate>
      </item>
    </channel></rss>"#;
    let feed = parse_feed(xml).unwrap();
    assert_eq!(feed.title, "Example");
    assert_eq!(feed.items.len(), 1);
    let item = &feed.items[0];
    assert_eq!(item.guid, "https://example.com/hello");
    assert!(item.content.contains("Hi"));
    assert!(!item.content.contains("script"));
    assert!(!item.content.contains("onclick"));
}

#[test]
fn falls_back_to_hash_guid_when_id_and_link_missing() {
    let xml = br#"<?xml version="1.0"?>
    <rss version="2.0"><channel>
      <title>Example</title>
      <item><title>No Link</title><description>Text</description></item>
    </channel></rss>"#;
    let feed = parse_feed(xml).unwrap();
    assert_eq!(feed.items[0].guid.len(), 64);
}
