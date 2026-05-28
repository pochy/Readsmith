use crate::error::{AppError, AppResult};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tokio::net::lookup_host;
use url::Url;

#[derive(Debug, Clone, Copy)]
pub struct UrlGuard {
    pub allow_private: bool,
}

impl UrlGuard {
    pub fn new(allow_private: bool) -> Self {
        Self { allow_private }
    }

    pub async fn validate_fetch_url(&self, raw: &str) -> AppResult<Url> {
        let url = self.validate_url_syntax(raw)?;
        if !self.allow_private {
            self.reject_private_host(&url).await?;
        }
        Ok(url)
    }

    pub fn validate_redirect_url(&self, raw: &str) -> AppResult<Url> {
        self.validate_url_syntax(raw)
    }

    fn validate_url_syntax(&self, raw: &str) -> AppResult<Url> {
        let url = Url::parse(raw.trim())
            .map_err(|e| AppError::BadRequest(format!("invalid URL: {e}")))?;
        match url.scheme() {
            "http" | "https" => {}
            other => {
                return Err(AppError::BadRequest(format!(
                    "unsupported URL scheme: {other}"
                )));
            }
        }
        if url.host_str().is_none() {
            return Err(AppError::BadRequest("URL host is required".to_string()));
        }
        Ok(url)
    }

    async fn reject_private_host(&self, url: &Url) -> AppResult<()> {
        let host = url
            .host_str()
            .ok_or_else(|| AppError::BadRequest("URL host is required".to_string()))?;
        if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
            return Err(AppError::BadRequest(
                "localhost feeds are disabled".to_string(),
            ));
        }
        if let Ok(ip) = host.parse::<IpAddr>() {
            reject_private_ip(ip)?;
            return Ok(());
        }
        let port = url.port_or_known_default().unwrap_or(80);
        let addrs = lookup_host((host, port))
            .await
            .map_err(|e| AppError::BadRequest(format!("cannot resolve feed host: {e}")))?;
        for addr in addrs {
            reject_private_ip(addr.ip())?;
        }
        Ok(())
    }
}

pub fn reject_private_ip(ip: IpAddr) -> AppResult<()> {
    if is_disallowed_ip(ip) {
        Err(AppError::BadRequest(format!(
            "private or local address is disabled: {ip}"
        )))
    } else {
        Ok(())
    }
}

pub fn is_disallowed_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_disallowed_v4(ip),
        IpAddr::V6(ip) => is_disallowed_v6(ip),
    }
}

fn is_disallowed_v4(ip: Ipv4Addr) -> bool {
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.octets()[0] == 0
        || ip.octets()[0] >= 224
}

fn is_disallowed_v6(ip: Ipv6Addr) -> bool {
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_unique_local()
        || ip.is_unicast_link_local()
        || ip.segments()[0] & 0xffc0 == 0xff80
}
