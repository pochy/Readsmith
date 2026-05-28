use readsmith::services::sanitizer::sanitize_html;

#[test]
fn removes_script_handlers_and_javascript_urls() {
    let sanitized = sanitize_html(
        r#"<p onclick="x()">ok</p><a href="javascript:alert(1)">bad</a><script>x()</script>"#,
    );
    assert!(sanitized.contains("ok"));
    assert!(!sanitized.contains("onclick"));
    assert!(!sanitized.contains("javascript:"));
    assert!(!sanitized.contains("<script"));
}
