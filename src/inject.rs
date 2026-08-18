// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Localhost slash inject via `POST /entrypoint/slash`.

use crate::health::{replace_health_path, PROBE_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Default inject URL derived from product health.
pub const DEFAULT_ENTRYPOINT_SLASH_URL: &str = "http://127.0.0.1:4700/entrypoint/slash";

#[derive(Serialize)]
struct SlashRequest {
    command: String,
    text: String,
}

#[derive(Deserialize)]
struct SlashResponse {
    #[serde(default)]
    reply: String,
}

/// POST `/list` and return the product reply text.
///
/// # Errors
///
/// Returns a short reason when the product is not ready or the HTTP call fails.
pub async fn fetch_saved_list(
    client: &reqwest::Client,
    health_url: &str,
) -> Result<String, String> {
    let url = replace_health_path(health_url, "/entrypoint/slash")
        .unwrap_or_else(|| DEFAULT_ENTRYPOINT_SLASH_URL.to_string());
    let resp = client
        .post(&url)
        .timeout(PROBE_TIMEOUT)
        .json(&SlashRequest {
            command: "/list".into(),
            text: String::new(),
        })
        .send()
        .await
        .map_err(|e| format!("request: {e}"))?;
    let status = resp.status().as_u16();
    if status == 503 {
        return Err("ITCy inject not ready".into());
    }
    if !(200..300).contains(&status) {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("http {status}: {body}"));
    }
    let parsed: SlashResponse = resp.json().await.map_err(|e| format!("parse: {e}"))?;
    Ok(parsed.reply)
}
