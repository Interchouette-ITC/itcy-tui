//! Runtime status probe (`GET /status`) for providers + routes + webhook wake.

use serde::Deserialize;
use std::time::Duration;

/// Default product status URL.
pub const DEFAULT_STATUS_URL: &str = "http://127.0.0.1:4700/status";

/// Last BAT / webhook wake snapshot from the product binary.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct BatWakeSnapshot {
    pub at_unix: i64,
    pub repo: String,
    pub pr_number: u64,
    pub reviewer: String,
    pub action: String,
    pub merged: bool,
    pub detail: String,
}

/// Provider pool + failover routes + S4h webhook fields from the always-on binary.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RuntimeStatus {
    pub providers: Vec<String>,
    pub freeform_route_head: String,
    pub freeform_route: String,
    pub draft_route_head: String,
    pub draft_route: String,
    #[serde(default)]
    pub github_webhook_configured: bool,
    #[serde(default)]
    pub last_bat_wake: Option<BatWakeSnapshot>,
}

impl RuntimeStatus {
    #[must_use]
    pub fn providers_csv(&self) -> String {
        if self.providers.is_empty() {
            "(empty)".into()
        } else {
            self.providers.join(", ")
        }
    }

    #[must_use]
    pub fn webhook_label(&self) -> &'static str {
        if self.github_webhook_configured {
            "ok"
        } else {
            "not configured"
        }
    }

    /// Detail line mirroring health: ready state + last wake summary.
    #[must_use]
    pub fn webhook_detail(&self) -> String {
        if !self.github_webhook_configured {
            return "GITHUB_WEBHOOK_SECRET unset; POST /github/webhook_ITCy returns 503".into();
        }
        match &self.last_bat_wake {
            None => "secret set; POST /github/webhook_ITCy ready · last wake never".into(),
            Some(_) => format!(
                "secret set; POST /github/webhook_ITCy ready · last wake {}",
                self.wake_summary()
            ),
        }
    }

    #[must_use]
    pub fn wake_summary(&self) -> String {
        match &self.last_bat_wake {
            None => "never".into(),
            Some(w) => {
                let pr = if w.pr_number == 0 {
                    "-".into()
                } else {
                    format!("#{}", w.pr_number)
                };
                let who = if w.reviewer.is_empty() {
                    "-"
                } else {
                    w.reviewer.as_str()
                };
                format!(
                    "{action} pr={pr} by={who} merged={} · {detail}",
                    w.merged,
                    action = w.action,
                    detail = w.detail
                )
            }
        }
    }
}

/// Fetches `/status`. Network / parse errors return `None`.
pub fn fetch_status(url: &str) -> Option<RuntimeStatus> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;
    let resp = client.get(url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RuntimeStatus {
        RuntimeStatus {
            providers: vec![],
            freeform_route_head: "(none)".into(),
            freeform_route: "(empty)".into(),
            draft_route_head: "(none)".into(),
            draft_route: "(empty)".into(),
            github_webhook_configured: false,
            last_bat_wake: None,
        }
    }

    #[test]
    fn providers_csv_empty() {
        assert_eq!(sample().providers_csv(), "(empty)");
    }

    #[test]
    fn wake_summary_never() {
        assert_eq!(sample().wake_summary(), "never");
    }

    #[test]
    fn wake_summary_with_snapshot() {
        let mut s = sample();
        s.github_webhook_configured = true;
        s.last_bat_wake = Some(BatWakeSnapshot {
            at_unix: 1,
            repo: "Interchouette-ITC/itcy-publications".into(),
            pr_number: 3,
            reviewer: "gRoussac".into(),
            action: "approved".into(),
            merged: true,
            detail: "merged PR #3 (rebase)".into(),
        });
        let summary = s.wake_summary();
        assert!(summary.contains("approved"));
        assert!(summary.contains("#3"));
        assert!(summary.contains("gRoussac"));
        assert_eq!(s.webhook_label(), "ok");
        let detail = s.webhook_detail();
        assert!(detail.contains("ready"));
        assert!(detail.contains("approved"));
    }

    #[test]
    fn webhook_detail_when_secret_missing() {
        assert!(sample()
            .webhook_detail()
            .contains("GITHUB_WEBHOOK_SECRET unset"));
    }
}
