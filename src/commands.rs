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

/// Authoritative TUI catalog (mirrors product help).
pub const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        usage: "help",
        summary: "this list",
        stub: false,
    },
    SlashCommand {
        usage: "status_itcy",
        summary: "process / routes / health snapshot",
        stub: false,
    },
    SlashCommand {
        usage: "/draft_about <subject>, <instructions>",
        summary: "draft; a https in instructions is the in-post cite",
        stub: false,
    },
    SlashCommand {
        usage: "/draft_about_itc",
        summary: "LinkedIn draft about Interchouette / our projects",
        stub: false,
    },
    SlashCommand {
        usage: "/rework <Draft-ID|Tweet-ID> <instructions>",
        summary: "rewrite saved draft or tweet",
        stub: false,
    },
    SlashCommand {
        usage: "/change_url <Draft-ID|Tweet-ID> <0|1|2|3|https://…>",
        summary: "set the link; 0 = no link",
        stub: false,
    },
    SlashCommand {
        usage: "/accept <Draft-ID|Tweet-ID>",
        summary: "open/update BAT PR (LinkedIn or X)",
        stub: false,
    },
    SlashCommand {
        usage: "/list",
        summary: "list saved drafts and tweets (not published)",
        stub: false,
    },
    SlashCommand {
        usage: "/show <Draft-ID|Tweet-ID>[, <ID>]",
        summary: "show saved draft(s) and/or tweet(s)",
        stub: false,
    },
    SlashCommand {
        usage: "/delete <Draft-ID|Tweet-ID>[, <ID>]",
        summary: "delete saved row(s) and close GitHub PRs",
        stub: false,
    },
    SlashCommand {
        usage: "/retry_bat <Draft-ID|Tweet-ID|XPOST-ID>",
        summary: "re-ship after BAT (missed webhook or ship failed)",
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
        summary: "press + follows + tweet searches + Interchouette into #daily-digest",
        stub: false,
    },
    SlashCommand {
        usage: "/propose_draft",
        summary: "new draft from corpus",
        stub: false,
    },
    SlashCommand {
        usage: "/tweet_about <subject>, <instructions>",
        summary: "tweet; https in instructions locks quote or link",
        stub: false,
    },
    SlashCommand {
        usage: "/tweet_farce",
        summary: "dad-joke tweet tagging @grok @cursor_ai @elonmusk",
        stub: false,
    },
    SlashCommand {
        usage: "/draft_tweet_about_itc",
        summary: "X tweet about Interchouette / our projects",
        stub: false,
    },
    SlashCommand {
        usage: "/propose_tweet",
        summary: "new tweet from corpus",
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
    "help",
    "status_itcy",
    "/draft_about",
    "/draft_about_itc",
    "/rework",
    "/change_url",
    "/accept",
    "/list",
    "/show",
    "/delete",
    "/retry_bat",
    "/enrich",
    "/ingest",
    "/daily_digest",
    "/propose_draft",
    "/tweet_about",
    "/tweet_farce",
    "/draft_tweet_about_itc",
    "/propose_tweet",
    "/accept_comment_reply",
];

/// Snapshot of product `help_text()` used to detect TUI catalog drift in tests.
pub const PRODUCT_HELP_TEXT_SNAPSHOT: &str = "\
ITCy runtime (`#itcy`).\n\
*Keywords (type in channel):*\n\
• `help` / `commands` - this list\n\
• `status_itcy` - process / routes / health snapshot\n\
*Slash workflows:*\n\
• `/draft_about <subject>, <instructions>` - draft; a https in instructions is the in-post cite\n\
• `/draft_about_itc` or `/draft_about_itc <subject>, <instructions>` - LinkedIn draft about Interchouette / our projects\n\
• `/rework <Draft-ID|Tweet-ID> <instructions>` - rewrite saved draft or tweet (until Post / XPOST)\n\
• `/change_url <Draft-ID|Tweet-ID> <0|1|2|3|https://…>` - set the link (`1`/`2`/`3` or URL); `0` = no link\n\
• `/accept <Draft-ID|Tweet-ID>` - open/update BAT PR (LinkedIn `drafts` or X `draft_tweet`; safe to re-run; publishes if Approve is on GitHub but webhook missed)\n\
• `/list` - list saved LinkedIn drafts and tweets (not published)\n\
• `/show <Draft-ID|Tweet-ID>[, <ID>]` - show saved draft(s) and/or tweet(s)\n\
• `/delete <Draft-ID|Tweet-ID>[, <ID>]` - delete saved row(s) and close GitHub PRs if open\n\
• `/retry_bat <Draft-ID|Tweet-ID|XPOST-ID>` - re-ship after BAT (missed webhook or X/LinkedIn ship failed)\n\
• `/enrich <url>` - enrich corpus with Greg LinkedIn post (Tor)\n\
• `/ingest <url>` - ingest public article or LinkedIn Pulse (clearnet)\n\
• `/daily_digest` - 20 press + 20 follows + 20 tweet searches + 10 Interchouette (5 draft / 5 tweet) into `#daily-digest`\n\
• `/propose_draft` - new draft from corpus (what we already know)\n\
• `/propose_draft <DIGEST-…>, <1|1,3>` or `/propose_draft <N>` - new drafts from that digest's propositions\n\
• `/tweet_about <subject>, <instructions>` - tweet; a https in instructions locks the quote (X status) or the link (publisher)\n\
• `/tweet_farce` or `/tweet_farce <theme hint>` - dad-joke / IT wordplay tagging @grok @cursor_ai @elonmusk (no cite)\n\
• `/draft_tweet_about_itc` or `/draft_tweet_about_itc <subject>, <instructions>` - X tweet about Interchouette / our projects\n\
• `/propose_tweet` - new tweet from corpus\n\
• `/propose_tweet <DIGEST-…>, <1|1,3>` or `/propose_tweet <N>` - new tweets from that digest's propositions\n\
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
