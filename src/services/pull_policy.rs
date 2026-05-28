use time::{Duration, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct PullInput {
    pub now: OffsetDateTime,
    pub next_check_at: Option<OffsetDateTime>,
    pub last_success_at: Option<OffsetDateTime>,
    pub last_error_at: Option<OffsetDateTime>,
    pub consecutive_failures: u32,
    pub cache_control: Option<String>,
    pub retry_after_until: Option<OffsetDateTime>,
    pub max_backoff_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullDecision {
    pub should_fetch: bool,
    pub next_check_at: OffsetDateTime,
    pub reason: String,
}

pub fn decide(input: PullInput) -> PullDecision {
    if let Some(retry_after) = input.retry_after_until {
        if retry_after > input.now {
            return PullDecision {
                should_fetch: false,
                next_check_at: retry_after,
                reason: "retry-after".to_string(),
            };
        }
    }
    if let Some(next) = input.next_check_at {
        if next > input.now {
            return PullDecision {
                should_fetch: false,
                next_check_at: next,
                reason: "not-due".to_string(),
            };
        }
    }
    PullDecision {
        should_fetch: true,
        next_check_at: next_after_fetch(&input),
        reason: if input.consecutive_failures > 0 {
            "due-after-error".to_string()
        } else {
            "due".to_string()
        },
    }
}

pub fn next_after_success(now: OffsetDateTime, cache_control: Option<&str>) -> OffsetDateTime {
    let seconds = cache_control
        .and_then(max_age)
        .unwrap_or(1800)
        .clamp(300, 86_400);
    now + Duration::seconds(seconds as i64)
}

pub fn next_after_error(
    now: OffsetDateTime,
    consecutive_failures: u32,
    max_backoff_seconds: u64,
) -> OffsetDateTime {
    let exponent = consecutive_failures.min(10);
    let seconds = 60u64
        .saturating_mul(2u64.saturating_pow(exponent))
        .min(max_backoff_seconds);
    now + Duration::seconds(seconds as i64)
}

fn next_after_fetch(input: &PullInput) -> OffsetDateTime {
    if input.consecutive_failures > 0 {
        next_after_error(
            input.now,
            input.consecutive_failures,
            input.max_backoff_seconds,
        )
    } else {
        next_after_success(input.now, input.cache_control.as_deref())
    }
}

pub fn max_age(cache_control: &str) -> Option<u64> {
    cache_control
        .split(',')
        .map(str::trim)
        .find_map(|part| part.strip_prefix("max-age=")?.parse().ok())
}
