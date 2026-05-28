use readsmith::services::pull_policy::{
    PullInput, decide, max_age, next_after_error, next_after_success,
};
use time::{Duration, OffsetDateTime};

#[test]
fn respects_retry_after_until() {
    let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    let retry_after = now + Duration::minutes(10);
    let decision = decide(PullInput {
        now,
        next_check_at: Some(now - Duration::seconds(1)),
        last_success_at: None,
        last_error_at: None,
        consecutive_failures: 3,
        cache_control: None,
        retry_after_until: Some(retry_after),
        max_backoff_seconds: 86_400,
    });
    assert!(!decision.should_fetch);
    assert_eq!(decision.next_check_at, retry_after);
}

#[test]
fn parses_cache_control_max_age() {
    assert_eq!(max_age("public, max-age=900"), Some(900));
    let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    assert_eq!(
        next_after_success(now, Some("max-age=120")).unix_timestamp(),
        (now + Duration::seconds(300)).unix_timestamp()
    );
}

#[test]
fn exponential_backoff_is_capped() {
    let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    assert_eq!(
        next_after_error(now, 1, 86_400),
        now + Duration::seconds(120)
    );
    assert_eq!(next_after_error(now, 20, 600), now + Duration::seconds(600));
}
