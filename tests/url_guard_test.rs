use readsmith::services::url_guard::{UrlGuard, is_disallowed_ip};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[tokio::test]
async fn rejects_non_http_schemes() {
    let guard = UrlGuard::new(false);
    let err = guard
        .validate_fetch_url("file:///etc/passwd")
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("unsupported URL scheme"));
}

#[test]
fn detects_private_and_local_ips() {
    assert!(is_disallowed_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
    assert!(is_disallowed_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    assert!(is_disallowed_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))));
    assert!(is_disallowed_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert!(!is_disallowed_ip(IpAddr::V4(Ipv4Addr::new(
        93, 184, 216, 34
    ))));
}

#[test]
fn validates_redirect_scheme_syntax() {
    let guard = UrlGuard::new(false);
    assert!(
        guard
            .validate_redirect_url("https://example.com/feed.xml")
            .is_ok()
    );
    assert!(guard.validate_redirect_url("gopher://example.com").is_err());
}
