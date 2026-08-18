// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! `:` command palette: prefix complete + history.

use crate::github::{PubsBranch, PubsRemote};
use crate::ui::ViewMode;

const COMMANDS: &[&str] = &[
    "live",
    "commands",
    "pubs",
    "list",
    "help",
    "org",
    "fork",
    "drafts",
    "posts",
    "drafts_tweet",
    "tweets",
    "reload",
    "open ",
];

/// Outcome of running a colon command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    /// Switch the main view.
    View(ViewMode),
    /// Org or fork.
    Remote(PubsRemote),
    /// Publications branch.
    Branch(PubsBranch),
    /// Refetch the current GitHub tree.
    Reload,
    /// Select first artefact whose id contains this prefix.
    Open(String),
    /// Unknown line.
    Unknown(String),
}

/// Prefix completions for the current buffer (cycle with Tab).
#[must_use]
pub fn completions(buffer: &str) -> Vec<String> {
    let t = buffer.trim_start().to_ascii_lowercase();
    COMMANDS
        .iter()
        .filter(|c| c.starts_with(&t) || (*c).starts_with("open ") && t.starts_with("open"))
        .map(|c| (*c).to_string())
        .collect()
}

/// Parse a submitted colon line.
#[must_use]
pub fn parse_command(line: &str) -> PaletteAction {
    let t = line.trim();
    let lower = t.to_ascii_lowercase();
    match lower.as_str() {
        "live" | "d" => PaletteAction::View(ViewMode::Live),
        "commands" | "c" => PaletteAction::View(ViewMode::Commands),
        "pubs" | "p" | "publications" => PaletteAction::View(ViewMode::Publications),
        "list" | "s" => PaletteAction::View(ViewMode::SavedList),
        "help" => PaletteAction::View(ViewMode::Help),
        "org" | "o" => PaletteAction::Remote(PubsRemote::Org),
        "fork" | "f" => PaletteAction::Remote(PubsRemote::Fork),
        "drafts" => PaletteAction::Branch(PubsBranch::Drafts),
        "posts" => PaletteAction::Branch(PubsBranch::Posts),
        "drafts_tweet" => PaletteAction::Branch(PubsBranch::DraftsTweet),
        "tweets" => PaletteAction::Branch(PubsBranch::Tweets),
        "reload" | "r" => PaletteAction::Reload,
        _ => lower.strip_prefix("open ").map_or_else(
            || PaletteAction::Unknown(t.to_string()),
            |rest| PaletteAction::Open(rest.trim().to_string()),
        ),
    }
}

/// Ring buffer of the last 32 submitted lines.
#[derive(Debug, Clone, Default)]
pub struct CommandHistory {
    lines: Vec<String>,
    cursor: Option<usize>,
}

impl CommandHistory {
    /// Remember a non-empty line (newest last). Drops duplicates then caps at 32.
    pub fn push(&mut self, line: &str) {
        let line = line.trim().to_string();
        if line.is_empty() {
            return;
        }
        self.lines.retain(|h| h != &line);
        self.lines.push(line);
        if self.lines.len() > 32 {
            self.lines.remove(0);
        }
        self.cursor = None;
    }

    /// Older entry, or `None` at the start.
    pub fn up(&mut self) -> Option<&str> {
        if self.lines.is_empty() {
            return None;
        }
        let next = match self.cursor {
            None => self.lines.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.cursor = Some(next);
        self.lines.get(next).map(String::as_str)
    }

    /// Newer entry, or `None` past the end (clears cursor).
    pub fn down(&mut self) -> Option<&str> {
        let i = self.cursor?;
        if i + 1 >= self.lines.len() {
            self.cursor = None;
            return None;
        }
        self.cursor = Some(i + 1);
        self.lines.get(i + 1).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_open_prefix() {
        assert_eq!(
            parse_command("open DRAFT-2026"),
            PaletteAction::Open("draft-2026".into())
        );
        assert_eq!(
            parse_command("pubs"),
            PaletteAction::View(ViewMode::Publications)
        );
    }

    #[test]
    fn complete_prefix() {
        let c = completions("dra");
        assert!(c.iter().any(|s| s == "drafts"));
        assert!(c.iter().any(|s| s == "drafts_tweet"));
    }

    #[test]
    fn history_cap_and_nav() {
        let mut h = CommandHistory::default();
        h.push("a");
        h.push("b");
        assert_eq!(h.up(), Some("b"));
        assert_eq!(h.up(), Some("a"));
        assert_eq!(h.down(), Some("b"));
        assert_eq!(h.down(), None);
    }
}
