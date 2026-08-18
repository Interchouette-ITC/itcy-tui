// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Keyboard and mouse help pane.

use super::model::StatusModel;
use super::style::pane_block;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

const HELP: &[&str] = &[
    "q / Ctrl-C                 quit",
    "Esc                        close overlay or help (never quits)",
    "d / c / p / s              live / commands / pubs / saved list",
    "?                          this help",
    ":                          command (Tab complete, Up/Down history)",
    "/                          filter current table",
    "r                          refresh probes; refetch tree on pubs; re-run /list",
    "y                          copy selected id (OSC 52)",
    "",
    "publications",
    "  o / f                    org / fork",
    "  1 2 3 4                  drafts / posts / drafts_tweet / tweets",
    "  Tab                      focus list or preview",
    "  j k  g G                 select / first / last",
    "  Enter                    load body now",
    "  click                    tabs, chips, rows, focus",
    "  wheel                    scroll focused pane",
    "",
    ":live :commands :pubs :list :help :org :fork :reload :open <id>",
    "",
    "commands: Enter on /list runs localhost inject (read). Writes stay in Slack.",
];

pub fn draw_help(frame: &mut Frame, area: Rect, model: &mut StatusModel) {
    model.hits.help_body = area;
    let lines: Vec<Line> = HELP
        .iter()
        .map(|l| {
            Line::from(Span::styled(
                (*l).to_string(),
                Style::default().fg(Color::Gray),
            ))
        })
        .collect();
    let body = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((model.help_scroll, 0))
        .block(pane_block("help", true));
    frame.render_widget(body, area);
}
