// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Parse `/list` reply lines into table rows.

/// One saved draft or tweet from the product `/list` text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedRow {
    /// `DRAFT-…` / `TWEET-…`.
    pub id: String,
    /// Store status (`open`, `building`, …).
    pub status: String,
    /// Clipped subject from the reply.
    pub subject: String,
}

/// Parse product `/list` bullets: id in backticks, then status, then subject.
#[must_use]
pub fn parse_saved_rows(reply: &str) -> Vec<SavedRow> {
    reply.lines().filter_map(parse_bullet).collect()
}

fn parse_bullet(line: &str) -> Option<SavedRow> {
    let t = line.trim();
    let rest = t.strip_prefix('•').unwrap_or(t).trim();
    let rest = rest.strip_prefix('`')?;
    let (id, after) = rest.split_once('`')?;
    let after = after.trim();
    let (status, subject) = after.split_once(" - ").unwrap_or((after, ""));
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(SavedRow {
        id: id.to_string(),
        status: status.trim().to_string(),
        subject: subject.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_product_bullet() {
        let rows = parse_saved_rows(
            "Drafts (1, newest first):\n• `DRAFT-20260817-000058` open - owl merge\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "DRAFT-20260817-000058");
        assert_eq!(rows[0].status, "open");
        assert_eq!(rows[0].subject, "owl merge");
    }
}
