// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Slash catalog table.

use super::model::StatusModel;
use super::style::{pane_block, MUTED};
use crate::commands::SLASH_COMMANDS;
use crate::health::HealthStatus;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table};
use ratatui::Frame;

pub fn draw_commands(frame: &mut Frame, area: Rect, model: &mut StatusModel) {
    model.hits.commands_table = area;
    let indexes = model.filtered_commands();
    let rows: Vec<Row> = indexes
        .iter()
        .map(|&idx| {
            let cmd = &SLASH_COMMANDS[idx];
            let stub = if cmd.stub { "(stub)" } else { "" };
            let list_ok = cmd.usage == "/list" && matches!(model.health, HealthStatus::Ok);
            let usage_style = if cmd.usage == "/list" && !list_ok {
                MUTED
            } else {
                Style::default().fg(Color::Cyan)
            };
            Row::new(vec![
                Cell::from(cmd.usage.to_string()).style(usage_style),
                Cell::from(stub.to_string()).style(Style::default().fg(Color::Yellow)),
                Cell::from(cmd.summary.to_string()).style(MUTED),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Min(28),
            Constraint::Length(8),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["usage", "flag", "summary"]).style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .row_highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("> ")
    .block(pane_block("commands", true));
    frame.render_stateful_widget(table, area, &mut model.cmd_state);
    let n = indexes.len();
    let mut scroll =
        ScrollbarState::new(n.max(1)).position(model.cmd_state.selected().unwrap_or(0));
    let bar = area.inner(ratatui::layout::Margin {
        horizontal: 0,
        vertical: 1,
    });
    if bar.width > 0 {
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            bar,
            &mut scroll,
        );
    }
}
