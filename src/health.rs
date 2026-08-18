// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Health probe against the `ITCy` always-on `/health` endpoint.

use std::time::Duration;

/// Default product health URL (see itcy `backend/config.toml`).
pub const DEFAULT_HEALTH_URL: &str = "http://127.0.0.1:4700/health";

/// Org webhook ingress (`itc-hooks`) health URL.
pub const DEFAULT_INGRESS_HEALTH_URL: &str = "http://127.0.0.1:7007/health";

/// HTTP client timeout for `/health` and `/status` probes.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Replaces a trailing `/health` path with `new_path` (e.g. `/status`, `/hooks/github`).
///
/// When `health_url` does not end with `/health`, returns `None` so the caller can fall back.
#[must_use]
pub fn replace_health_path(health_url: &str, new_path: &str) -> Option<String> {
    health_url
        .ends_with("/health")
        .then(|| health_url.replacen("/health", new_path, 1))
}

/// Result of probing the product `/health` endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Body was `ok` with HTTP 200.
    Ok,
    /// Probe failed or body was not `ok`.
    Down {
        /// Short reason for the pane.
        reason: String,
    },
}

impl HealthStatus {
    /// Short label for the status pane (`ok` or `DOWN`).
    #[must_use]
    pub const fn label(&self) -> &str {
        match self {
            Self::Ok => "ok",
            Self::Down { .. } => "DOWN",
        }
    }
}

/// Maps an HTTP status + body to a health status (product returns plain `ok`).
#[must_use]
pub fn interpret_health(status: u16, body: &str) -> HealthStatus {
    if status == 200 && body.trim() == "ok" {
        HealthStatus::Ok
    } else {
        HealthStatus::Down {
            reason: format!("status={status} body={}", body.trim()),
        }
    }
}

/// GETs `url` and interprets the response. Network errors become `Down`.
#[must_use]
pub fn fetch_health(url: &str) -> HealthStatus {
    let client = match reqwest::blocking::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return HealthStatus::Down {
                reason: format!("client: {e}"),
            };
        }
    };

    match client.get(url).send() {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().unwrap_or_default();
            interpret_health(status, &body)
        }
        Err(e) => HealthStatus::Down {
            reason: format!("request: {e}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpret_ok_body() {
        assert_eq!(interpret_health(200, "ok"), HealthStatus::Ok);
        assert_eq!(interpret_health(200, "ok\n"), HealthStatus::Ok);
    }

    #[test]
    fn interpret_rejects_wrong_status_or_body() {
        assert!(matches!(
            interpret_health(500, "ok"),
            HealthStatus::Down { .. }
        ));
        assert!(matches!(
            interpret_health(200, "ready"),
            HealthStatus::Down { .. }
        ));
    }

    #[test]
    fn replace_health_path_swaps_suffix() {
        assert_eq!(
            replace_health_path(DEFAULT_HEALTH_URL, "/status").as_deref(),
            Some("http://127.0.0.1:4700/status")
        );
        assert_eq!(
            replace_health_path("http://127.0.0.1:4700/", "/status"),
            None
        );
    }
}
