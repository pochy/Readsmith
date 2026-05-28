use crate::{config::Config, domain::user_session::UserSession};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use time::{Duration, OffsetDateTime};

type HmacSha256 = Hmac<Sha256>;

pub const COOKIE_NAME: &str = "readsmith_session";

pub fn verify_password(config: &Config, password: &str) -> bool {
    match &config.password {
        Some(expected) => constant_time_eq(expected.as_bytes(), password.as_bytes()),
        None => config.allow_empty_password,
    }
}

pub fn make_session_cookie(config: &Config) -> String {
    let exp = (OffsetDateTime::now_utc() + Duration::days(30)).unix_timestamp();
    let payload = format!("v1.{exp}");
    let sig = sign(config, &payload);
    let secure = if config.secure_cookies {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{COOKIE_NAME}={payload}.{sig}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000{secure}"
    )
}

pub fn expire_session_cookie() -> String {
    format!("{COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

pub fn parse_session(config: &Config, cookie_header: Option<&str>) -> Option<UserSession> {
    let raw = cookie_header?.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == COOKIE_NAME).then_some(value)
    })?;
    let mut parts = raw.split('.');
    let version = parts.next()?;
    let exp = parts.next()?.parse::<i64>().ok()?;
    let sig = parts.next()?;
    if version != "v1" || parts.next().is_some() {
        return None;
    }
    let payload = format!("v1.{exp}");
    if !constant_time_eq(sign(config, &payload).as_bytes(), sig.as_bytes()) {
        return None;
    }
    if exp <= OffsetDateTime::now_utc().unix_timestamp() {
        return None;
    }
    Some(UserSession { expires_at: exp })
}

fn sign(config: &Config, payload: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(config.session_secret.as_bytes()).expect("HMAC accepts any key");
    mac.update(payload.as_bytes());
    URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
