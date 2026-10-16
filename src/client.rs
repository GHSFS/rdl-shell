//! Minimal `POST /api/dl` client. Mirrors the surface used by `rdl-tray`.

use crate::config::Config;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
use url::Url;

const USER_AGENT: &str = concat!("rdl-shell/", env!("CARGO_PKG_VERSION"));

static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

fn get_client() -> &'static reqwest::blocking::Client {
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .https_only(true)
            .tcp_nodelay(true)
            .build()
            .unwrap_or_else(|e| {
                eprintln!("rdl-shell: warning: failed to build custom client: {e}");
                reqwest::blocking::Client::new()
            })
    })
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JobRef {
    pub id: String,
}

#[derive(Debug, Serialize)]
struct QueueRequest<'a> {
    url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder: Option<&'a str>,
}

/// Validates that the provided URL has a valid syntax and an allowed scheme (http or https).
pub fn validate_url(raw: &str) -> Result<Url> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(Error::InvalidUrl("URL string is empty".to_string()));
    }
    let parsed = Url::parse(trimmed).map_err(|e| Error::InvalidUrl(e.to_string()))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        other => Err(Error::InvalidUrl(format!(
            "unsupported URL scheme: {other}"
        ))),
    }
}

pub fn queue(cfg: &Config, url: &str) -> Result<JobRef> {
    validate_url(url)?;

    let http = get_client();

    let body = QueueRequest {
        url: url.trim(),
        folder: cfg.folder.as_deref(),
    };
    let resp = http
        .post(format!("{}/api/dl", cfg.worker()?))
        .header("Authorization", format!("Bearer {}", cfg.token()?))
        .json(&body)
        .send()
        .map_err(|e| Error::Http(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(match status.as_u16() {
            401 | 403 => Error::Auth(format!("server rejected credentials ({status})")),
            _ => Error::Http(format!("unexpected status {status}")),
        });
    }
    resp.json::<JobRef>()
        .map_err(|e| Error::Http(e.to_string()))
}

/// Queues all provided URLs sequentially using the pooled HTTP client.
pub fn queue_all(cfg: &Config, urls: &[String]) -> Vec<Result<JobRef>> {
    urls.iter().map(|u| queue(cfg, u)).collect()
}
