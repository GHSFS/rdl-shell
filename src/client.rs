//! Minimal `POST /api/dl` client. Mirrors the surface used by `rdl-tray`.

use crate::config::Config;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const USER_AGENT: &str = concat!("rdl-shell/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize)]
pub struct JobRef {
    pub id: String,
}

#[derive(Debug, Serialize)]
struct QueueRequest<'a> {
    url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder: Option<&'a str>,
}

pub fn queue(cfg: &Config, url: &str) -> Result<JobRef> {
    let http = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .https_only(true)
        .build()
        .map_err(|e| Error::Http(e.to_string()))?;

    let body = QueueRequest {
        url,
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
    resp.json::<JobRef>().map_err(|e| Error::Http(e.to_string()))
}
