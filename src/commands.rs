//! Static slash-command catalog for the TUI reference pane.
//!
//! Keep in sync with product `help_text()` in `backend/crates/itcy/src/slack/commands.rs`.

/// One operator slash command shown in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlashCommand {
    /// Usage line, e.g. `/ingest <external url>`.
    pub usage: &'static str,
    /// Short summary.
    pub summary: &'static str,
    /// True when the product still stubs the command (S6/S8).
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
        usage: "/change_draft_url <Draft-ID> <1|2|3|url>",
        summary: "swap in-post link",
        stub: false,
    },
    SlashCommand {
        usage: "/accept_draft <Draft-ID>",
        summary: "publications BAT PR (zero LinkedIn live)",
        stub: false,
    },
    SlashCommand {
        usage: "/enrich <linkedin post url>",
        summary: "enrich corpus with Greg LinkedIn post (Tor)",
        stub: false,
    },
    SlashCommand {
        usage: "/ingest <external url>",
        summary: "ingest public article into corpus",
        stub: false,
    },
    SlashCommand {
        usage: "/propose_draft",
        summary: "ITCy proposes subject + draft",
        stub: true,
    },
    SlashCommand {
        usage: "/accept_comment_reply <post url>",
        summary: "accept comment-reply BAT pack",
        stub: true,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_ingest_external_url() {
        assert!(SLASH_COMMANDS
            .iter()
            .any(|c| c.usage.contains("/ingest <external url>")));
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
