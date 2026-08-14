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
        summary: "draft from corpus about a topic you name",
        stub: false,
    },
    SlashCommand {
        usage: "/rework_draft <Draft-ID> <instructions>",
        summary: "rewrite saved draft (works until Post)",
        stub: false,
    },
    SlashCommand {
        usage: "/change_draft_url <Draft-ID> <1|2|3|https://…>",
        summary: "swap in-post link (works until Post)",
        stub: false,
    },
    SlashCommand {
        usage: "/accept_draft <Draft-ID>",
        summary: "open/update fork Draft PR",
        stub: false,
    },
    SlashCommand {
        usage: "/list_drafts",
        summary: "list saved LinkedIn drafts (not published)",
        stub: false,
    },
    SlashCommand {
        usage: "/show_draft <Draft-ID>",
        summary: "show one saved draft",
        stub: false,
    },
    SlashCommand {
        usage: "/delete_draft <Draft-ID>",
        summary: "delete a saved draft and close its GitHub PR",
        stub: false,
    },
    SlashCommand {
        usage: "/retry_bat <Draft-ID|Tweet-ID>",
        summary: "Approve landed, webhook missed → publish",
        stub: false,
    },
    SlashCommand {
        usage: "/enrich <url>",
        summary: "enrich corpus with Greg LinkedIn post (Tor)",
        stub: false,
    },
    SlashCommand {
        usage: "/ingest <url>",
        summary: "ingest public article or LinkedIn Pulse (clearnet)",
        stub: false,
    },
    SlashCommand {
        usage: "/daily_digest",
        summary: "press + follows + tweet searches into #daily-digest",
        stub: false,
    },
    SlashCommand {
        usage: "/propose_draft",
        summary: "new draft from corpus",
        stub: false,
    },
    SlashCommand {
        usage: "/tweet_about <subject>, <instructions>",
        summary: "tweet from corpus (publisher URL or X quote)",
        stub: false,
    },
    SlashCommand {
        usage: "/propose_tweet",
        summary: "new tweet from corpus",
        stub: false,
    },
    SlashCommand {
        usage: "/rework_tweet <Tweet-ID>, <instructions>",
        summary: "rewrite saved tweet (works until XPOST)",
        stub: false,
    },
    SlashCommand {
        usage: "/change_tweet_url <Tweet-ID>, <1|2|3|https://…>",
        summary: "swap cite (publisher or X status)",
        stub: false,
    },
    SlashCommand {
        usage: "/accept_tweet <Tweet-ID>",
        summary: "open/update fork PR into draft_tweet",
        stub: false,
    },
    SlashCommand {
        usage: "/list_tweets",
        summary: "list saved tweets (not published)",
        stub: false,
    },
    SlashCommand {
        usage: "/show_tweet <Tweet-ID>",
        summary: "show one saved tweet",
        stub: false,
    },
    SlashCommand {
        usage: "/delete_tweet <Tweet-ID>",
        summary: "delete a saved tweet and close its GitHub PR",
        stub: false,
    },
    SlashCommand {
        usage: "/accept_comment_reply <https://…>",
        summary: "LinkedIn comment-reply BAT (not wired yet)",
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
    "/list_drafts",
    "/show_draft",
    "/delete_draft",
    "/retry_bat",
    "/enrich",
    "/ingest",
    "/daily_digest",
    "/propose_draft",
    "/tweet_about",
    "/propose_tweet",
    "/rework_tweet",
    "/change_tweet_url",
    "/accept_tweet",
    "/list_tweets",
    "/show_tweet",
    "/delete_tweet",
    "/accept_comment_reply",
];

/// Snapshot of product `help_text()` used to detect TUI/Slack catalog drift in tests.
pub const PRODUCT_HELP_TEXT_SNAPSHOT: &str = "\
ITCy runtime (`#itcy`).\n\
*Slash workflows:*\n\
• `/help` - this list\n\
• `/status_itcy` - process / routes / health snapshot\n\
• `/draft_about <subject>, <instructions>` - draft from corpus about a topic you name\n\
• `/rework_draft <Draft-ID> <instructions>` - rewrite saved draft (works until Post)\n\
• `/change_draft_url <Draft-ID> <1|2|3|https://…>` - swap in-post link (works until Post)\n\
• `/accept_draft <Draft-ID>` - open/update fork Draft PR (safe to re-run if already accepted; publishes Post if Approve is on GitHub but webhook missed)\n\
• `/list_drafts` - list saved LinkedIn drafts (not published)\n\
• `/show_draft <Draft-ID>` - show one saved draft\n\
• `/delete_draft <Draft-ID>` - delete a saved draft and close its GitHub PR if open\n\
• `/retry_bat <Draft-ID|Tweet-ID>` - same: Approve already landed, webhook missed → publish Post or XPOST\n\
• `/enrich <url>` - enrich corpus with Greg LinkedIn post (Tor)\n\
• `/ingest <url>` - ingest public article or LinkedIn Pulse (clearnet)\n\
• `/daily_digest` - 20 press + 20 follows + 20 tweet searches into `#daily-digest` (not corpus, not LinkedIn)\n\
• `/propose_draft` - new draft from corpus (what we already know)\n\
• `/propose_draft <DIGEST-…>, <1|1,3>` or `/propose_draft <N>` - new drafts from that digest's propositions\n\
• `/tweet_about <subject>, <instructions>` - tweet from corpus (cite = publisher URL or X status quote)\n\
• `/propose_tweet` - new tweet from corpus\n\
• `/propose_tweet <DIGEST-…>, <1|1,3>` or `/propose_tweet <N>` - new tweets from that digest's propositions\n\
• `/rework_tweet <Tweet-ID>, <instructions>` - rewrite saved tweet (works until XPOST)\n\
• `/change_tweet_url <Tweet-ID>, <1|2|3|https://…>` - swap cite (publisher or X status)\n\
• `/accept_tweet <Tweet-ID>` - open/update fork PR into draft_tweet\n\
• `/list_tweets` - list saved tweets (not published)\n\
• `/show_tweet <Tweet-ID>` - show one saved tweet\n\
• `/delete_tweet <Tweet-ID>` - delete a saved tweet and close its GitHub PR if open\n\
• `/accept_comment_reply <https://…>` - LinkedIn comment-reply BAT (not wired yet; not a tweet reply)\n\
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
    fn stubs_are_comment_reply_only() {
        let stubs: Vec<_> = SLASH_COMMANDS.iter().filter(|c| c.stub).collect();
        assert_eq!(stubs.len(), 1);
        assert!(stubs
            .iter()
            .any(|c| c.usage.starts_with("/accept_comment_reply")));
    }
}
