use readsmith::{
    domain::{feed::FeedWithGroup, group::Group},
    services::opml,
};

#[test]
fn parses_grouped_opml_feeds() {
    let xml = br#"<?xml version="1.0"?>
    <opml version="2.0"><body>
      <outline text="Tech">
        <outline text="Example" title="Example" type="rss" xmlUrl="https://example.com/feed.xml" htmlUrl="https://example.com"/>
      </outline>
    </body></opml>"#;
    let feeds = opml::parse_import(xml).unwrap();
    assert_eq!(feeds.len(), 1);
    assert_eq!(feeds[0].group_name, "Tech");
    assert_eq!(feeds[0].xml_url, "https://example.com/feed.xml");
}

#[test]
fn exports_grouped_opml() {
    let groups = vec![Group {
        id: 1,
        name: "Default".to_string(),
        created_at: 0,
        updated_at: 0,
    }];
    let feeds = vec![FeedWithGroup {
        id: 1,
        group_id: 1,
        group_name: "Default".to_string(),
        title: "Example".to_string(),
        feed_url: "https://example.com/feed.xml".to_string(),
        site_url: "https://example.com".to_string(),
        description: String::new(),
        suspended: 0,
        unread_count: 0,
    }];
    let xml = opml::export(&groups, &feeds);
    assert!(xml.contains("readsmith.opml").not());
    assert!(xml.contains("xmlUrl=\"https://example.com/feed.xml\""));
}

trait BoolNot {
    fn not(self) -> bool;
}

impl BoolNot for bool {
    fn not(self) -> bool {
        !self
    }
}
