// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Parse publication artefact paths (flat, legacy YYYY/MM, and YYYY/MM/DD shards).

/// One artefact folder on a publications branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artefact {
    /// Folder id (`DRAFT-…`, `POST-…`, `TWEET-…`, `XPOST-…`).
    pub id: String,
    /// `YYYY/MM` or `YYYY/MM/DD` when the tree is sharded.
    pub shard: Option<String>,
    /// Path to `body.md`.
    pub body_path: String,
    /// Sibling `meta.toml`.
    pub meta_path: String,
    /// Subject from `meta.toml` once loaded this session.
    pub subject: String,
}

/// `body.md` path -> artefact, or `None` if not a known id folder.
#[must_use]
pub fn artefact_from_body_path(path: &str) -> Option<Artefact> {
    let name = path.replace('\\', "/");
    if !name.ends_with("/body.md") {
        return None;
    }
    let id = name.split('/').find(|seg| is_artefact_id(seg))?.to_string();
    let meta_path = format!("{}meta.toml", name.trim_end_matches("body.md"));
    Some(Artefact {
        shard: shard_prefix(&name),
        body_path: name,
        meta_path,
        id,
        subject: String::new(),
    })
}

fn is_artefact_id(seg: &str) -> bool {
    seg.starts_with("DRAFT-")
        || seg.starts_with("POST-")
        || seg.starts_with("TWEET-")
        || seg.starts_with("XPOST-")
}

fn shard_prefix(path: &str) -> Option<String> {
    let mut parts = path.split('/');
    let year = parts.next()?;
    let month = parts.next()?;
    if year.len() != 4
        || !year.bytes().all(|b| b.is_ascii_digit())
        || month.len() != 2
        || !month.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let Some(day) = parts.next() else {
        return Some(format!("{year}/{month}"));
    };
    if day.len() == 2 && day.bytes().all(|b| b.is_ascii_digit()) {
        Some(format!("{year}/{month}/{day}"))
    } else {
        Some(format!("{year}/{month}"))
    }
}

/// Best-effort `subject = "…"` from pack `meta.toml`.
#[must_use]
pub fn subject_from_meta(meta: &str) -> String {
    for line in meta.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix("subject =") else {
            continue;
        };
        let rest = rest.trim();
        if let Some(inner) = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            return inner.replace("\\\"", "\"").replace("\\\\", "\\");
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_and_sharded() {
        let flat = artefact_from_body_path("TWEET-20260813-000001/body.md").expect("flat");
        assert_eq!(flat.id, "TWEET-20260813-000001");
        assert_eq!(flat.shard, None);
        assert_eq!(flat.meta_path, "TWEET-20260813-000001/meta.toml");

        let shard =
            artefact_from_body_path("2026/08/13/XPOST-20260813-000001/body.md").expect("shard");
        assert_eq!(shard.id, "XPOST-20260813-000001");
        assert_eq!(shard.shard.as_deref(), Some("2026/08/13"));
        assert_eq!(
            shard.meta_path,
            "2026/08/13/XPOST-20260813-000001/meta.toml"
        );

        let legacy =
            artefact_from_body_path("2026/08/XPOST-20260813-000001/body.md").expect("legacy");
        assert_eq!(legacy.shard.as_deref(), Some("2026/08"));

        let draft = artefact_from_body_path("DRAFT-20260801-000001/body.md").expect("draft");
        assert_eq!(draft.id, "DRAFT-20260801-000001");
        assert!(artefact_from_body_path("README.md").is_none());
        assert!(artefact_from_body_path("2026/08/TWEET-20260813-000001/meta.toml").is_none());
    }

    #[test]
    fn subject_line() {
        assert_eq!(
            subject_from_meta("kind = \"tweet\"\nsubject = \"owl merge\"\n"),
            "owl merge"
        );
        assert_eq!(subject_from_meta("kind = \"tweet\"\n"), "");
    }
}
