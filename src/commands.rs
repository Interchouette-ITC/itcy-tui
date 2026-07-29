// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Static slash-command catalog for the TUI reference pane.
//!
//! Keep in sync with product `help_text()` in `backend/crates/itcy/src/slack/commands.rs`.

/// One operator slash command shown in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlashCommand {
    /// Usage line, e.g. `/ingest <url>`.
    pub usage: &'static str,
    /// Short summary.
    pub summary: &'static str,
    /// True when the product still stubs the command.
    pub stub: bool,
}

/// Authoritative TUI catalog (mirrors Slack `/help`).
pub const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        usage: "/help",
        summary: "this list",
        stub: false,
    },
    SlashCommand {
        usage: "/status_itcy",
        summary: "process / routes / health snapshot",
        stub: false,
    },
    SlashCommand {
        usage: "/draft_about <subject>, <instructions>",
        summary: "grounded LinkedIn draft (LOAD + writer)",
        stub: false,
    },
    SlashCommand {
        usage: "/rework_draft <Draft-ID> <instructions>",
        summary: "rewrite same draft",
        stub: false,
    },
    SlashCommand {
        usage: "/change_draft_url <Draft-ID> <1|2|3|https://…>",
        summary: "swap in-post link",
        stub: false,
    },
    SlashCommand {
        usage: "/accept_draft <Draft-ID>",
        summary: "publications BAT PR (zero LinkedIn live)",
        stub: false,
    },
    SlashCommand {
        usage: "/enrich <url>",
        summary: "enrich corpus with Greg LinkedIn post (Tor)",
        stub: false,
    },
    SlashCommand {
        usage: "/ingest <url>",
        summary: "ingest public article into corpus",
        stub: false,
    },
    SlashCommand {
        usage: "/propose_draft",
        summary: "ITCy proposes subject + draft (not wired yet)",
        stub: true,
    },
    SlashCommand {
        usage: "/accept_comment_reply <https://…>",
        summary: "accept comment-reply BAT pack (not wired yet)",
        stub: true,
    },
];

/// Command name prefixes that must appear in product `help_text()` (sync check).
pub const HELP_TEXT_COMMAND_PREFIXES: &[&str] = &[
    "/help",
    "/status_itcy",
    "/draft_about",
    "/rework_draft",
    "/change_draft_url",
    "/accept_draft",
    "/enrich",
    "/ingest",
    "/propose_draft",
    "/accept_comment_reply",
];

/// Snapshot of product `help_text()` used to detect TUI/Slack catalog drift in tests.
pub const PRODUCT_HELP_TEXT_SNAPSHOT: &str = "\
ITCy runtime (`#itcy`).\n\
*Slash workflows:*\n\
• `/help` - this list\n\
• `/status_itcy` - process / routes / health snapshot\n\
• `/draft_about <subject>, <instructions>` - grounded LinkedIn draft (LOAD + writer)\n\
• `/rework_draft <Draft-ID> <instructions>` - rewrite same draft\n\
• `/change_draft_url <Draft-ID> <1|2|3|https://…>` - swap in-post link\n\
• `/accept_draft <Draft-ID>` - publications BAT PR (zero LinkedIn live)\n\
• `/enrich <url>` - enrich corpus with Greg LinkedIn post (Tor)\n\
• `/ingest <url>` - ingest public article into corpus\n\
• `/propose_draft` - ITCy proposes subject + draft (not wired yet)\n\
• `/accept_comment_reply <https://…>` - accept comment-reply BAT pack (not wired yet)\n\
*Freeform chat:* anything else (informal / informational; tools OK). No draft/BAT/corpus ingest here.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_matches_help_prefixes() {
        for prefix in HELP_TEXT_COMMAND_PREFIXES {
            assert!(
                SLASH_COMMANDS.iter().any(|c| c.usage.starts_with(prefix)),
                "TUI catalog missing {prefix}"
            );
            assert!(
                PRODUCT_HELP_TEXT_SNAPSHOT.contains(prefix),
                "help snapshot missing {prefix}"
            );
        }
        assert_eq!(SLASH_COMMANDS.len(), HELP_TEXT_COMMAND_PREFIXES.len());
    }

    #[test]
    fn catalog_summaries_align_with_help_snapshot() {
        for cmd in SLASH_COMMANDS {
            let name = cmd.usage.split_whitespace().next().unwrap();
            assert!(
                PRODUCT_HELP_TEXT_SNAPSHOT.contains(name),
                "help snapshot missing {name}"
            );
            // Stub rows must say not wired; live rows must not.
            if cmd.stub {
                assert!(
                    cmd.summary.contains("not wired yet"),
                    "stub {} needs not-wired summary",
                    cmd.usage
                );
            }
        }
    }

    #[test]
    fn stubs_are_propose_and_comment_reply() {
        let stubs: Vec<_> = SLASH_COMMANDS.iter().filter(|c| c.stub).collect();
        assert_eq!(stubs.len(), 2);
        assert!(stubs.iter().any(|c| c.usage.starts_with("/propose_draft")));
        assert!(stubs
            .iter()
            .any(|c| c.usage.starts_with("/accept_comment_reply")));
    }
}
