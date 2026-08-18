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
pub fn fetch_saved_list(health_url: &str) -> Result<String, String> {
    let url = replace_health_path(health_url, "/entrypoint/slash")
        .unwrap_or_else(|| DEFAULT_ENTRYPOINT_SLASH_URL.to_string());
    let client = reqwest::blocking::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()
        .map_err(|e| format!("client: {e}"))?;
    let resp = client
        .post(&url)
        .json(&SlashRequest {
            command: "/list".into(),
            text: String::new(),
        })
        .send()
        .map_err(|e| format!("request: {e}"))?;
    let status = resp.status().as_u16();
    if status == 503 {
        return Err("ITCy inject not ready".into());
    }
    if !(200..300).contains(&status) {
        let body = resp.text().unwrap_or_default();
        return Err(format!("http {status}: {body}"));
    }
    let parsed: SlashResponse = resp.json().map_err(|e| format!("parse: {e}"))?;
    Ok(parsed.reply)
}
