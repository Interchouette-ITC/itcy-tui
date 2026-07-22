//! Health probe against the ITCy always-on `/health` endpoint.

use std::time::Duration;

/// Default product health URL (see itcy `backend/config.toml`).
pub const DEFAULT_HEALTH_URL: &str = "http://127.0.0.1:4700/health";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Ok,
    Down { reason: String },
}

impl HealthStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Ok => "ok",
            Self::Down { .. } => "DOWN",
        }
    }
}

/// Maps an HTTP status + body to a health status (product returns plain `ok`).
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
pub fn fetch_health(url: &str) -> HealthStatus {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
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
}
