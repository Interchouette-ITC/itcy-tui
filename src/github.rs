// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Read-only GitHub client for public `itcy-publications` trees.

use crate::artefact::{artefact_from_body_path, Artefact};
use crate::health::PROBE_TIMEOUT;
use serde::Deserialize;
use std::time::Duration;

/// Org publications owner.
pub const ORG_OWNER: &str = "Interchouette-ITC";
/// Worker-fork publications owner.
pub const FORK_OWNER: &str = "Interchouette";
/// Publications repo name.
pub const PUBS_REPO: &str = "itcy-publications";

const TREE_TIMEOUT: Duration = Duration::from_secs(20);

/// Org vs fork remote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PubsRemote {
    /// `Interchouette-ITC/itcy-publications`.
    Org,
    /// `Interchouette/itcy-publications`.
    Fork,
}

impl PubsRemote {
    /// GitHub owner login.
    #[must_use]
    pub const fn owner(self) -> &'static str {
        match self {
            Self::Org => ORG_OWNER,
            Self::Fork => FORK_OWNER,
        }
    }

    /// Short label for tabs.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Org => "org",
            Self::Fork => "fork",
        }
    }
}

/// Publications branch = kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PubsBranch {
    /// `drafts` branch.
    Drafts,
    /// `posts` branch.
    Posts,
    /// `drafts_tweet` branch.
    DraftsTweet,
    /// `tweets` branch.
    Tweets,
}

impl PubsBranch {
    /// Git ref name.
    #[must_use]
    pub const fn git_name(self) -> &'static str {
        match self {
            Self::Drafts => "drafts",
            Self::Posts => "posts",
            Self::DraftsTweet => "drafts_tweet",
            Self::Tweets => "tweets",
        }
    }

    /// Short label for tabs.
    #[must_use]
    pub const fn label(self) -> &'static str {
        self.git_name()
    }

    /// Digit keys: `1` drafts, `2` posts, `3` `drafts_tweet`, `4` tweets.
    #[must_use]
    pub const fn from_digit(digit: char) -> Option<Self> {
        match digit {
            '1' => Some(Self::Drafts),
            '2' => Some(Self::Posts),
            '3' => Some(Self::DraftsTweet),
            '4' => Some(Self::Tweets),
            _ => None,
        }
    }

    /// All kinds in tab order.
    pub const ALL: [Self; 4] = [Self::Drafts, Self::Posts, Self::DraftsTweet, Self::Tweets];
}

/// Tree fetch outcome.
#[derive(Debug, Clone)]
pub struct TreeFetch {
    /// Artefacts with a `body.md`.
    pub artefacts: Vec<Artefact>,
    /// GitHub `X-RateLimit-Remaining`, if present.
    pub rate_remaining: Option<u32>,
    /// Error line when the request failed (artefacts empty).
    pub error: Option<String>,
}

/// Body + meta for one artefact.
#[derive(Debug, Clone)]
pub struct PreviewFetch {
    /// Artefact id.
    pub id: String,
    /// `body.md` text or error.
    pub body: Result<String, String>,
    /// Subject from `meta.toml`.
    pub subject: String,
}

#[derive(Deserialize)]
struct GitTree {
    #[serde(default)]
    tree: Vec<GitTreeEntry>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct GitTreeEntry {
    path: String,
    #[serde(rename = "type")]
    kind: String,
}

fn optional_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn apply_auth(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    match optional_token() {
        Some(t) => req.bearer_auth(t),
        None => req,
    }
}

fn rate_from(resp: &reqwest::Response) -> Option<u32> {
    resp.headers()
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
}

/// Recursive git tree for `owner/itcy-publications` at `branch`.
#[must_use]
pub async fn fetch_branch_tree(
    client: &reqwest::Client,
    remote: PubsRemote,
    branch: PubsBranch,
) -> TreeFetch {
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
        remote.owner(),
        PUBS_REPO,
        branch.git_name()
    );
    let resp = match apply_auth(client.get(&url).timeout(TREE_TIMEOUT))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return TreeFetch {
                artefacts: Vec::new(),
                rate_remaining: None,
                error: Some(format!("request: {e}")),
            };
        }
    };
    let rate_remaining = rate_from(&resp);
    let status = resp.status().as_u16();
    let parsed: GitTree = match resp.json().await {
        Ok(p) => p,
        Err(e) => {
            return TreeFetch {
                artefacts: Vec::new(),
                rate_remaining,
                error: Some(format!("parse tree: {e}")),
            };
        }
    };
    if status == 403 {
        return TreeFetch {
            artefacts: Vec::new(),
            rate_remaining,
            error: Some("rate limited or forbidden".into()),
        };
    }
    if status == 404 {
        return TreeFetch {
            artefacts: Vec::new(),
            rate_remaining,
            error: Some(format!("branch `{}` not found", branch.git_name())),
        };
    }
    if !(200..300).contains(&status) {
        let msg = parsed.message.unwrap_or_else(|| format!("http {status}"));
        return TreeFetch {
            artefacts: Vec::new(),
            rate_remaining,
            error: Some(msg),
        };
    }
    let mut artefacts: Vec<Artefact> = parsed
        .tree
        .into_iter()
        .filter(|e| e.kind == "blob")
        .filter_map(|e| artefact_from_body_path(&e.path))
        .collect();
    artefacts.sort_by(|a, b| b.id.cmp(&a.id));
    TreeFetch {
        artefacts,
        rate_remaining,
        error: None,
    }
}

/// File text (`body.md` / `meta.toml`) at `path` on `branch`.
///
/// # Errors
///
/// Returns a short reason when the HTTP client fails or GitHub responds non-2xx.
pub async fn fetch_file_text(
    client: &reqwest::Client,
    remote: PubsRemote,
    branch: PubsBranch,
    path: &str,
) -> Result<String, String> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{path}",
        remote.owner(),
        PUBS_REPO,
        branch.git_name()
    );
    let resp = apply_auth(client.get(&url).timeout(PROBE_TIMEOUT))
        .send()
        .await
        .map_err(|e| format!("request: {e}"))?;
    let status = resp.status().as_u16();
    if status == 404 {
        return Err("file not found".into());
    }
    if !(200..300).contains(&status) {
        return Err(format!("http {status}"));
    }
    resp.text().await.map_err(|e| format!("body: {e}"))
}

/// Load `body.md` and subject for one artefact.
#[must_use]
pub async fn fetch_preview(
    client: &reqwest::Client,
    remote: PubsRemote,
    branch: PubsBranch,
    id: String,
    body_path: String,
    meta_path: String,
) -> PreviewFetch {
    let body = fetch_file_text(client, remote, branch, &body_path).await;
    let subject = fetch_file_text(client, remote, branch, &meta_path)
        .await
        .ok()
        .map(|m| crate::artefact::subject_from_meta(&m))
        .unwrap_or_default();
    PreviewFetch { id, body, subject }
}
