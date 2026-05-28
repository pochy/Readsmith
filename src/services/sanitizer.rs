use ammonia::Builder;
use std::collections::HashSet;

pub fn sanitize_html(input: &str) -> String {
    let tags: HashSet<&str> = [
        "a",
        "abbr",
        "blockquote",
        "br",
        "code",
        "div",
        "em",
        "figure",
        "figcaption",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "hr",
        "img",
        "li",
        "ol",
        "p",
        "pre",
        "span",
        "strong",
        "table",
        "tbody",
        "td",
        "th",
        "thead",
        "tr",
        "ul",
    ]
    .into_iter()
    .collect();
    let attrs: HashSet<&str> = ["href", "src", "alt", "title", "width", "height"]
        .into_iter()
        .collect();
    Builder::default()
        .tags(tags)
        .generic_attributes(attrs)
        .url_schemes(["http", "https", "mailto"].into_iter().collect())
        .clean(input)
        .to_string()
}
