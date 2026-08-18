// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Saved `/list` table.

use super::model::StatusModel;
use super::style::{pane_block, MUTED};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
};
use ratatui::Frame;

pub fn draw_saved(frame: &mut Frame, area: Rect, model: &mut StatusModel) {
    model.hits.saved_table = area;
    if !model.saved_error.is_empty() && model.saved_rows.is_empty() {
        let body = Paragraph::new(Line::from(Span::styled(
            model.saved_error.clone(),
            Style::default().fg(Color::LightRed),
        )))
        .block(pane_block("saved /list", true));
        frame.render_widget(body, area);
        return;
    }
    let indexes = model.filtered_saved();
    let rows: Vec<Row> = indexes
        .iter()
        .map(|&idx| {
            let r = &model.saved_rows[idx];
            Row::new(vec![
                Cell::from(r.id.clone()),
                Cell::from(r.status.clone()),
                Cell::from(r.subject.clone()).style(MUTED),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Min(22),
            Constraint::Length(10),
            Constraint::Min(12),
        ],
    )
    .header(
        Row::new(vec!["id", "status", "subject"]).style(
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
    .block(pane_block("saved /list", true));
    frame.render_stateful_widget(table, area, &mut model.saved_state);
    let n = indexes.len();
    let mut scroll =
        ScrollbarState::new(n.max(1)).position(model.saved_state.selected().unwrap_or(0));
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
