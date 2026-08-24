// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Ratatui chrome: tabs, overlay, footer, views.

mod commands_table;
mod help;
mod hits;
mod live;
mod model;
mod pubs;
mod saved;
mod style;

pub use hits::{contains, HitMap};
pub use model::{InputMode, IoNeed, PubsFocus, PubsPane, StatusModel};

use crate::health::HealthStatus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;
use style::{pane_block, tab_span, ACCENT, LABEL, MUTED};

/// Which pane the TUI is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Live health / routes / webhook pane.
    #[default]
    Live,
    /// Slash-command reference pane.
    Commands,
    /// Public GitHub publications browser.
    Publications,
    /// Localhost `/list` table.
    SavedList,
    /// Keys and mouse help.
    Help,
}

/// Draw the shell into `frame` (records mouse hits on `model`).
pub fn draw(frame: &mut Frame, model: &mut StatusModel) {
    let overlay_h = u16::from(model.overlay_open()) * 3;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(overlay_h),
            Constraint::Length(2),
        ])
        .split(frame.area());
    draw_title(frame, chunks[0], model);
    match model.view {
        ViewMode::Live => live::draw_live(frame, chunks[1], model),
        ViewMode::Commands => commands_table::draw_commands(frame, chunks[1], model),
        ViewMode::Publications => pubs::draw_publications(frame, chunks[1], model),
        ViewMode::SavedList => saved::draw_saved(frame, chunks[1], model),
        ViewMode::Help => help::draw_help(frame, chunks[1], model),
    }
    if overlay_h > 0 {
        draw_overlay(frame, chunks[2], model);
    }
    draw_footer(frame, chunks[3], model);
}

fn draw_title(frame: &mut Frame, area: Rect, model: &mut StatusModel) {
    let inner = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            "itcy-tui",
            ACCENT.add_modifier(Modifier::BOLD),
        ))
        .inner(area);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                "itcy-tui",
                ACCENT.add_modifier(Modifier::BOLD),
            )),
        area,
    );
    let tabs = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(4),
        ])
        .split(inner);
    let specs = [
        (tabs[0], "d:live", ViewMode::Live),
        (tabs[1], "c:commands", ViewMode::Commands),
        (tabs[2], "p:pubs", ViewMode::Publications),
        (tabs[3], "s:list", ViewMode::SavedList),
    ];
    model.hits.live_tab = tabs[0];
    model.hits.commands_tab = tabs[1];
    model.hits.pubs_tab = tabs[2];
    model.hits.list_tab = tabs[3];
    for (rect, label, mode) in specs {
        frame.render_widget(
            Paragraph::new(Line::from(tab_span(label, model.view == mode))),
            rect,
        );
    }
}

fn draw_overlay(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let title = match model.input {
        InputMode::Filter => {
            let count = model
                .match_count()
                .map_or_else(|| "filter".into(), |(n, t)| format!("filter {n}/{t}"));
            count
        }
        InputMode::Command => "command".into(),
        InputMode::Normal => String::new(),
    };
    let block = pane_block(&title, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(model.overlay_line()), inner);
    if inner.width > 0 && inner.height > 0 {
        let x = inner.x.saturating_add(model.textarea.cursor_col());
        let max_x = inner.x.saturating_add(inner.width.saturating_sub(1));
        frame.set_cursor_position((x.min(max_x), inner.y));
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let mode = if matches!(model.health, HealthStatus::Ok) {
        "local"
    } else {
        "public"
    };
    let view = match model.view {
        ViewMode::Live => "live",
        ViewMode::Commands => "commands",
        ViewMode::Publications => "pubs",
        ViewMode::SavedList => "list",
        ViewMode::Help => "help",
    };
    let (status, style) = if !model.notice.is_empty() {
        (model.notice.clone(), Style::default().fg(Color::LightRed))
    } else if model.copied {
        ("copied".into(), Style::default().fg(Color::LightGreen))
    } else if model.pending {
        let spin = ["|", "/", "-", "\\"];
        let i = usize::try_from(model.ticks).unwrap_or(0) % spin.len();
        (
            format!("{} loading", spin[i]),
            Style::default().fg(Color::Yellow),
        )
    } else {
        (String::new(), MUTED)
    };
    let id = model.yank_text().unwrap_or_default();
    let keys = footer_keys(model.view);
    let mut spans = vec![
        Span::styled(" ", LABEL),
        Span::styled(mode, ACCENT.add_modifier(Modifier::BOLD)),
        Span::styled(" · ", MUTED),
        Span::styled(view, ACCENT.add_modifier(Modifier::BOLD)),
    ];
    if model.view == ViewMode::Publications {
        if let Some(rate) = model.pubs.rate_remaining {
            spans.push(Span::styled(format!(" · rate {rate}"), MUTED));
        }
    }
    if !status.is_empty() {
        spans.push(Span::styled(" · ", MUTED));
        spans.push(Span::styled(status, style));
    }
    if !id.is_empty() {
        spans.push(Span::styled(" · ", MUTED));
        spans.push(Span::styled(id, ACCENT.add_modifier(Modifier::BOLD)));
    }
    if let Some((n, total)) = model.match_count() {
        if model.filter_active() {
            spans.push(Span::styled(" · ", MUTED));
            spans.push(Span::styled(format!("{n}/{total}"), MUTED));
        }
    }
    spans.push(Span::styled(" · ", MUTED));
    spans.push(Span::styled(keys, MUTED));
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

const fn footer_keys(view: ViewMode) -> &'static str {
    match view {
        ViewMode::Live => "d c p s  ?  :  /  q",
        ViewMode::Commands => "j k  Enter=/list  y  /  q",
        ViewMode::Publications => "o f  1-4  Tab  j k  y  /  :  q",
        ViewMode::SavedList => "j k  y  r  /  q",
        ViewMode::Help => "Esc back  q quit",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artefact::Artefact;
    use crate::health::DEFAULT_HEALTH_URL;
    use crate::status::DEFAULT_STATUS_URL;
    use crate::status::{
        GithubDeliverySnapshot, LinkedInMcpSnapshot, RuntimeStatus, TorListenSnapshot,
    };
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn sample_runtime() -> RuntimeStatus {
        RuntimeStatus {
            providers: vec!["ollama".into()],
            freeform_route_head: "ollama:gemma4:12b".into(),
            freeform_route: "ollama:gemma4:12b, ollama:llama3.1:8b".into(),
            load_route_head: "ollama:llama3.1:8b".into(),
            load_route: "ollama:llama3.1:8b, ollama:gemma3:4b".into(),
            draft_route_head: "ollama:gemma4:12b".into(),
            draft_route: "ollama:gemma4:12b".into(),
            github_webhook_configured: true,
            last_bat_wake: None,
            last_github_delivery: Some(GithubDeliverySnapshot {
                at_unix: 1,
                event: "ping".into(),
                delivery_id: "d1".into(),
                outcome: "ok".into(),
                http_status: 200,
            }),
            github_delivery_warn: None,
            enrich: Some(crate::status::sample_enrich()),
            tor: Some(TorListenSnapshot {
                ok: true,
                socks_ok: true,
                control_ok: true,
                detail: "ok".into(),
            }),
            linkedin_mcp: Some(LinkedInMcpSnapshot {
                ok: true,
                url: "http://127.0.0.1:4780/mcp".into(),
                listen_addr: "127.0.0.1:4780".into(),
                detail: "ok".into(),
            }),
        }
    }

    fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
        buffer
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect()
    }

    fn fixture_artefact(id: &str, shard: Option<&str>) -> Artefact {
        let folder = shard.map_or_else(|| id.to_string(), |s| format!("{s}/{id}"));
        Artefact {
            id: id.to_string(),
            shard: shard.map(ToString::to_string),
            body_path: format!("{folder}/body.md"),
            meta_path: format!("{folder}/meta.toml"),
            subject: String::new(),
        }
    }

    #[test]
    fn render_shows_ok_and_providers() {
        let backend = TestBackend::new(100, 32);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
            HealthStatus::Ok,
            Some(sample_runtime()),
        );
        terminal.draw(|f| draw(f, &mut model)).expect("draw");
        let flat = buffer_text(terminal.backend().buffer());
        assert!(flat.contains("ok"), "missing ok: {flat}");
        assert!(flat.contains("d:live"), "missing live tab: {flat}");
        assert!(flat.contains("providers"), "missing providers");
        assert!(flat.contains("ollama"), "missing ollama");
        assert!(flat.contains("/hooks/github"), "missing webhook url");
        assert!(flat.contains("remaining=209"), "missing queue: {flat}");
        assert!(
            !flat.contains("showcase"),
            "footer must not say showcase: {flat}"
        );
    }

    #[test]
    fn render_commands_table_shows_ingest() {
        let backend = TestBackend::new(100, 36);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
            HealthStatus::Ok,
            None,
        );
        model.show_commands();
        terminal.draw(|f| draw(f, &mut model)).expect("draw");
        let flat = buffer_text(terminal.backend().buffer());
        assert!(flat.contains("/ingest"), "missing /ingest: {flat}");
        assert!(flat.contains("c:commands"), "missing commands tab: {flat}");
    }

    #[test]
    fn render_publications_fixture() {
        let backend = TestBackend::new(120, 28);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
            HealthStatus::Down {
                reason: "connection refused".into(),
            },
            None,
        );
        model.view = ViewMode::Publications;
        model.pubs.artefacts = vec![
            fixture_artefact("DRAFT-20260801-000001", None),
            fixture_artefact("TWEET-20260813-000001", Some("2026/08/13")),
        ];
        model.pubs.body = "hello vitrine body".into();
        model.pubs.artefacts[0].subject = "owl merge".into();
        terminal.draw(|f| draw(f, &mut model)).expect("draw");
        let flat = buffer_text(terminal.backend().buffer());
        assert!(
            flat.contains("DRAFT-20260801-000001"),
            "missing draft: {flat}"
        );
        assert!(flat.contains("2026/08/13"), "missing shard: {flat}");
        assert!(flat.contains("hello vitrine body"), "missing body: {flat}");
        assert!(flat.contains("p:pubs"), "missing pubs tab: {flat}");
        assert!(flat.contains("o:org"), "missing org chip: {flat}");
        assert!(flat.contains("1:drafts"), "missing branch chip: {flat}");
        assert!(flat.contains("public"), "missing public mode: {flat}");
        assert!(flat.contains('>'), "missing highlight: {flat}");
    }

    #[test]
    fn filter_matches_id_and_subject() {
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
            HealthStatus::Down {
                reason: "down".into(),
            },
            None,
        );
        model.pubs.artefacts = vec![
            fixture_artefact("DRAFT-20260801-000001", None),
            fixture_artefact("TWEET-20260813-000001", Some("2026/08/13")),
        ];
        model.pubs.artefacts[1].subject = "owl merge".into();
        model.pubs.set_filter("owl".into());
        let idx = model.pubs.filtered_indexes();
        assert_eq!(idx.len(), 1);
        assert_eq!(model.pubs.artefacts[idx[0]].id, "TWEET-20260813-000001");
    }

    #[test]
    fn boot_public_skips_fetch_when_tree_present() {
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
            HealthStatus::Down {
                reason: "down".into(),
            },
            None,
        );
        model.pubs.artefacts = vec![fixture_artefact("DRAFT-20260801-000001", None)];
        let need = model.apply_boot_view();
        assert_eq!(model.view, ViewMode::Publications);
        assert_eq!(need, IoNeed::None);
    }

    #[test]
    fn render_saved_list_when_itcy_down() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
            HealthStatus::Down {
                reason: "connection refused".into(),
            },
            None,
        );
        let need = model.show_saved_list();
        assert_eq!(need, IoNeed::None);
        assert_eq!(model.view, ViewMode::SavedList);
        terminal.draw(|f| draw(f, &mut model)).expect("draw");
        let flat = buffer_text(terminal.backend().buffer());
        assert!(flat.contains("/list needs"), "missing down message: {flat}");
    }

    #[test]
    fn render_help_and_filter_overlay() {
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
            HealthStatus::Ok,
            None,
        );
        model.show_help();
        terminal.draw(|f| draw(f, &mut model)).expect("draw");
        let flat = buffer_text(terminal.backend().buffer());
        assert!(flat.contains("quit"), "missing help: {flat}");
        model.leave_help();
        model.view = ViewMode::Publications;
        model.begin_filter();
        model.textarea.insert_str("DRAFT");
        model.sync_filter_from_overlay();
        terminal.draw(|f| draw(f, &mut model)).expect("draw");
        let flat = buffer_text(terminal.backend().buffer());
        assert!(flat.contains("filter"), "missing overlay: {flat}");
        assert!(flat.contains("0/0") || flat.contains("filter"), "{flat}");
    }

    #[test]
    fn render_warn_delivery() {
        let backend = TestBackend::new(80, 32);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut runtime = sample_runtime();
        runtime.github_delivery_warn = Some("HMAC reject".into());
        runtime.last_github_delivery = Some(GithubDeliverySnapshot {
            at_unix: 1,
            event: "push".into(),
            delivery_id: "d2".into(),
            outcome: "reject_hmac".into(),
            http_status: 401,
        });
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
            HealthStatus::Ok,
            Some(runtime),
        );
        terminal.draw(|f| draw(f, &mut model)).expect("draw");
        let flat = buffer_text(terminal.backend().buffer());
        assert!(flat.contains("WARN"), "missing WARN: {flat}");
        assert!(flat.contains("HMAC reject"), "missing warn text: {flat}");
    }

    #[test]
    fn open_prefix_selects() {
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
            HealthStatus::Ok,
            None,
        );
        model.pubs.artefacts = vec![
            fixture_artefact("DRAFT-20260801-000001", None),
            fixture_artefact("TWEET-20260813-000001", Some("2026/08/13")),
        ];
        assert!(model.pubs.open_prefix("tweet"));
        assert_eq!(
            model.pubs.selected_artefact().map(|a| a.id.as_str()),
            Some("TWEET-20260813-000001")
        );
    }
}
