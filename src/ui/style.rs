// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Shared styles for the TUI chrome.

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

pub const LABEL: Style = Style::new().fg(Color::DarkGray);
pub const ACCENT: Style = Style::new().fg(Color::Cyan);
pub const MUTED: Style = Style::new().fg(Color::Gray);

#[must_use]
pub fn pane_block(title: &str, focused: bool) -> Block<'static> {
    let border = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(ratatui::text::Span::styled(
            title.to_string(),
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ))
}

#[must_use]
pub fn tab_span(label: &str, active: bool) -> ratatui::text::Span<'static> {
    let style = if active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    ratatui::text::Span::styled(format!(" {label} "), style)
}
