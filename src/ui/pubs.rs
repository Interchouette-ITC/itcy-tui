// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Publications browser: chips + table + preview.

use super::model::{PubsFocus, StatusModel};
use super::style::{pane_block, tab_span, ACCENT, MUTED};
use crate::github::PubsBranch;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, Wrap,
};
use ratatui::Frame;

pub fn draw_publications(frame: &mut Frame, area: Rect, model: &mut StatusModel) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(area);
    draw_chips(frame, rows[0], model);
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(rows[1]);
    draw_list(frame, cols[0], model);
    draw_preview(frame, cols[1], model);
}

fn draw_chips(frame: &mut Frame, area: Rect, model: &mut StatusModel) {
    let inner = pane_block("kind", false).inner(area);
    frame.render_widget(pane_block("kind", false), area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(2),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(16),
            Constraint::Min(8),
        ])
        .split(inner);
    let pane = &model.pubs;
    let org = tab_span("o:org", pane.remote.label() == "org");
    let fork = tab_span("f:fork", pane.remote.label() == "fork");
    frame.render_widget(Paragraph::new(Line::from(org)), chunks[0]);
    frame.render_widget(Paragraph::new(Line::from(fork)), chunks[1]);
    model.hits.org_chip = chunks[0];
    model.hits.fork_chip = chunks[1];
    let labels = ["1:drafts", "2:posts", "3:drafts_tweet", "4:tweets"];
    for (i, branch) in PubsBranch::ALL.iter().enumerate() {
        let span = tab_span(labels[i], pane.branch == *branch);
        frame.render_widget(Paragraph::new(Line::from(span)), chunks[i + 3]);
        model.hits.branch_chips[i] = chunks[i + 3];
    }
}

fn draw_list(frame: &mut Frame, area: Rect, model: &mut StatusModel) {
    model.hits.pubs_list = area;
    let focused = model.pubs.focus == PubsFocus::List;
    let indexes = model.pubs.filtered_indexes();
    let rows: Vec<Row> = indexes
        .iter()
        .map(|&idx| {
            let art = &model.pubs.artefacts[idx];
            let shard = art.shard.as_deref().unwrap_or("-");
            let subject = if art.subject.is_empty() {
                "-"
            } else {
                art.subject.as_str()
            };
            Row::new(vec![
                Cell::from(art.id.clone()),
                Cell::from(shard.to_string()),
                Cell::from(subject.to_string()).style(MUTED),
            ])
        })
        .collect();
    let n = rows.len();
    let table = Table::new(
        rows,
        [
            Constraint::Min(22),
            Constraint::Length(8),
            Constraint::Min(12),
        ],
    )
    .header(
        Row::new(vec!["id", "shard", "subject"]).style(
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
    .block(pane_block("articles", focused));
    frame.render_stateful_widget(table, area, &mut model.pubs.table);
    let mut scroll = ScrollbarState::new(n.max(1)).position(model.pubs.selected());
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

fn draw_preview(frame: &mut Frame, area: Rect, model: &mut StatusModel) {
    model.hits.pubs_preview = area;
    let focused = model.pubs.focus == PubsFocus::Preview;
    let pane = &model.pubs;
    let id = pane
        .selected_artefact()
        .map_or_else(|| "(none)".into(), |a| a.id.clone());
    let mut lines = vec![Line::from(vec![
        Span::styled("id  ", MUTED),
        Span::styled(id, ACCENT.add_modifier(Modifier::BOLD)),
    ])];
    if !pane.error.is_empty() {
        lines.push(Line::from(Span::styled(
            pane.error.clone(),
            Style::default().fg(Color::LightRed),
        )));
    }
    lines.push(Line::from(""));
    if pane.body.is_empty() {
        lines.push(Line::from(Span::styled(
            "Enter loads body  ·  wait 400ms after j/k  ·  y yank id",
            MUTED,
        )));
    } else {
        for body_line in pane.body.lines() {
            lines.push(Line::from(Span::raw(body_line.to_string())));
        }
    }
    let n = lines.len();
    let preview = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((pane.body_scroll, 0))
        .block(pane_block("preview", focused));
    frame.render_widget(preview, area);
    let mut scroll = ScrollbarState::new(n.max(1)).position(usize::from(pane.body_scroll));
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
