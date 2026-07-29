// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Ratatui widgets for the ITCy status pane.

use crate::commands::SLASH_COMMANDS;
use crate::health::HealthStatus;
use crate::status::RuntimeStatus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

const LABEL: Style = Style::new().fg(Color::DarkGray);
const ACCENT: Style = Style::new().fg(Color::Cyan);
const MUTED: Style = Style::new().fg(Color::Gray);

/// Which pane the TUI is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Live,
    Commands,
}

#[derive(Debug, Clone)]
pub struct StatusModel {
    pub health_url: String,
    pub status_url: String,
    pub webhook_url: String,
    pub health: HealthStatus,
    pub runtime: Option<RuntimeStatus>,
    pub ticks: u64,
    pub view: ViewMode,
}

impl StatusModel {
    pub fn new(
        health_url: impl Into<String>,
        status_url: impl Into<String>,
        health: HealthStatus,
        runtime: Option<RuntimeStatus>,
    ) -> Self {
        let health_url = health_url.into();
        let webhook_url = hooks_url_from_health(&health_url);
        Self {
            health_url,
            status_url: status_url.into(),
            webhook_url,
            health,
            runtime,
            ticks: 0,
            view: ViewMode::Live,
        }
    }

    /// Toggle live status ↔ slash-command reference.
    pub fn toggle_commands(&mut self) {
        self.view = match self.view {
            ViewMode::Live => ViewMode::Commands,
            ViewMode::Commands => ViewMode::Live,
        };
    }
}

/// Same host as `/health`, path `POST /github/webhook_ITCy`.
fn hooks_url_from_health(health_url: &str) -> String {
    if health_url.ends_with("/health") {
        health_url.replacen("/health", "/github/webhook_ITCy", 1)
    } else {
        format!("{}/github/webhook_ITCy", health_url.trim_end_matches('/'))
    }
}

/// Draws the status pane into `frame`.
pub fn draw(frame: &mut Frame, model: &StatusModel) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(frame.area());

    draw_title(frame, chunks[0]);
    match model.view {
        ViewMode::Live => draw_body(frame, chunks[1], model),
        ViewMode::Commands => draw_commands(frame, chunks[1]),
    }
    draw_footer(frame, chunks[2], model);
}

fn draw_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "ITCy",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ·  ", MUTED),
        Span::styled(
            "Interchouette ITC status",
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                "itcy-tui",
                ACCENT.add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(title, area);
}

fn labeled(label: &str, value: impl AsRef<str>, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<11}"), LABEL),
        Span::styled(value.as_ref().to_string(), value_style),
    ])
}

fn draw_body(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let (health_color, detail, detail_style) = match &model.health {
        HealthStatus::Ok => (
            Color::LightGreen,
            "product /health answered ok".to_string(),
            Style::default().fg(Color::Green),
        ),
        HealthStatus::Down { reason } => (
            Color::LightRed,
            reason.clone(),
            Style::default().fg(Color::Red),
        ),
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("{:<11}", "health:"), LABEL),
            Span::styled(
                model.health.label().to_string(),
                Style::default()
                    .fg(health_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        labeled("url:", &model.health_url, ACCENT),
        labeled("detail:", &detail, detail_style),
        Line::from(""),
    ];

    match &model.runtime {
        Some(rt) => {
            lines.push(labeled(
                "providers:",
                rt.providers_csv(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
            lines.push(labeled(
                "freeform:",
                format!("head={} | {}", rt.freeform_route_head, rt.freeform_route),
                Style::default().fg(Color::LightBlue),
            ));
            lines.push(labeled(
                "load:",
                format!("head={} | {}", rt.load_route_head, rt.load_route),
                Style::default().fg(Color::LightCyan),
            ));
            lines.push(labeled(
                "draft:",
                format!("head={} | {}", rt.draft_route_head, rt.draft_route),
                Style::default().fg(Color::LightBlue),
            ));
            lines.push(Line::from(""));
            let webhook_color = if rt.github_webhook_configured {
                Color::LightGreen
            } else {
                Color::LightYellow
            };
            let webhook_detail = rt.webhook_detail();
            let webhook_detail_style = if rt.github_webhook_configured {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{:<11}", "webhook:"), LABEL),
                Span::styled(
                    rt.webhook_label().to_string(),
                    Style::default()
                        .fg(webhook_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(labeled("url:", &model.webhook_url, ACCENT));
            lines.push(labeled("detail:", &webhook_detail, webhook_detail_style));

            let delivery_label = rt.delivery_label();
            let delivery_color = match delivery_label {
                "ok" => Color::LightGreen,
                "WARN" => Color::LightRed,
                _ => Color::LightYellow,
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{:<11}", "delivery:"), LABEL),
                Span::styled(
                    delivery_label.to_string(),
                    Style::default()
                        .fg(delivery_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(labeled(
                "last:",
                rt.delivery_detail(),
                Style::default().fg(Color::Gray),
            ));
            if let Some(warn) = rt.delivery_warn_line() {
                lines.push(labeled(
                    "warn:",
                    warn,
                    Style::default().fg(Color::LightYellow),
                ));
            }
            lines.push(Line::from(""));
            match &rt.enrich {
                Some(en) => {
                    let enrich_color = match en.enrich_label() {
                        "running" => Color::LightGreen,
                        "idle" => Color::Gray,
                        _ => Color::LightYellow,
                    };
                    lines.push(Line::from(vec![
                        Span::styled(format!("{:<11}", "enrich:"), LABEL),
                        Span::styled(
                            en.enrich_label().to_string(),
                            Style::default()
                                .fg(enrich_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                    lines.push(labeled(
                        "counts:",
                        en.enrich_detail(),
                        Style::default().fg(Color::LightCyan),
                    ));
                    lines.push(labeled(
                        "queue:",
                        en.queue_detail(),
                        Style::default().fg(Color::LightBlue),
                    ));
                    lines.push(labeled(
                        "wall:",
                        en.wall_detail(),
                        Style::default().fg(Color::Gray),
                    ));
                }
                None => {
                    lines.push(labeled(
                        "enrich:",
                        "(unavailable)",
                        Style::default().fg(Color::Yellow),
                    ));
                }
            }
        }
        None => {
            lines.push(labeled(
                "routes:",
                format!("(unavailable - GET {})", model.status_url),
                Style::default().fg(Color::Yellow),
            ));
        }
    }

    lines.push(Line::from(""));
    lines.push(keys_line());
    lines.push(Line::from(vec![
        Span::styled("logs:  ", LABEL),
        Span::styled(
            "product screen window `itcy` (RUST_LOG) - not this TUI",
            MUTED,
        ),
    ]));

    let body = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(Span::styled(
                "live",
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(body, area);
}

fn draw_commands(frame: &mut Frame, area: Rect) {
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Slack #itcy slash workflows",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Freeform chat: anything else (no draft/BAT/corpus ingest)",
            MUTED,
        )]),
        Line::from(""),
    ];

    for cmd in SLASH_COMMANDS {
        let mut spans = vec![Span::styled(
            cmd.usage.to_string(),
            ACCENT.add_modifier(Modifier::BOLD),
        )];
        if cmd.stub {
            spans.push(Span::styled("  (stub)", Style::default().fg(Color::Yellow)));
        }
        spans.push(Span::styled(format!("  - {}", cmd.summary), MUTED));
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));
    lines.push(keys_line());

    let body = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(Span::styled(
                "slash commands",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(body, area);
}

fn keys_line() -> Line<'static> {
    Line::from(vec![
        Span::styled("keys:  ", LABEL),
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" / Esc / Ctrl-C  quit", MUTED),
        Span::styled("   ·   ", MUTED),
        Span::styled(
            "r",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  refresh", MUTED),
        Span::styled("   ·   ", MUTED),
        Span::styled(
            "c",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  commands", MUTED),
    ])
}

fn draw_footer(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("polls: ", LABEL),
        Span::styled(
            model.ticks.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   |   ", MUTED),
        Span::styled("Written by AI", Style::default().fg(Color::Yellow)),
        Span::styled(" - ", MUTED),
        Span::styled("ITCy", ACCENT.add_modifier(Modifier::BOLD)),
        Span::styled(" - ratatui", MUTED),
    ]));
    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;
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
            last_github_delivery: Some(crate::status::GithubDeliverySnapshot {
                at_unix: 1,
                event: "ping".into(),
                delivery_id: "d1".into(),
                outcome: "ok".into(),
                http_status: 200,
            }),
            github_delivery_warn: None,
            enrich: Some(crate::status::EnrichStatusSnapshot {
                pending: 171,
                in_flight: 0,
                ok: 85,
                failed: 1,
                skip: 1,
                none: 37,
                queue_remaining: 209,
                next_enrich_after: Some("2026-07-29T05:22:35+02:00".into()),
                wall_streak: Some(0),
                last_wall_source_id: None,
                enrich_pid: Some(2666262),
                enrich_running: true,
            }),
        }
    }

    #[test]
    fn render_shows_ok_and_providers() {
        let backend = TestBackend::new(80, 32);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let model = StatusModel::new(
            "http://127.0.0.1:4700/health",
            "http://127.0.0.1:4700/status",
            HealthStatus::Ok,
            Some(sample_runtime()),
        );
        terminal.draw(|f| draw(f, &model)).expect("draw");
        let buffer = terminal.backend().buffer().clone();
        let flat: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(flat.contains("ok"), "buffer missing ok: {flat}");
        assert!(flat.contains("providers"), "buffer missing providers");
        assert!(flat.contains("ollama"), "buffer missing ollama");
        assert!(flat.contains("webhook"), "buffer missing webhook");
        assert!(
            flat.contains("/github/webhook_ITCy"),
            "buffer missing webhook url: {flat}"
        );
        assert!(flat.contains("ready"), "buffer missing webhook detail");
        assert!(flat.contains("delivery"), "buffer missing delivery");
        assert!(flat.contains("ping"), "buffer missing last delivery event");
        assert!(flat.contains("enrich"), "buffer missing enrich");
        assert!(
            flat.contains("remaining=209"),
            "buffer missing queue: {flat}"
        );
        assert!(flat.contains("wall"), "buffer missing wall");
        assert!(
            !flat.contains("showcase"),
            "footer must not say showcase: {flat}"
        );
    }

    #[test]
    fn render_commands_pane_shows_ingest() {
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut model = StatusModel::new(
            "http://127.0.0.1:4700/health",
            "http://127.0.0.1:4700/status",
            HealthStatus::Ok,
            None,
        );
        model.toggle_commands();
        assert_eq!(model.view, ViewMode::Commands);
        terminal.draw(|f| draw(f, &model)).expect("draw");
        let buffer = terminal.backend().buffer().clone();
        let flat: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(flat.contains("/ingest"), "missing /ingest: {flat}");
        assert!(
            flat.contains("external url"),
            "missing external url usage: {flat}"
        );
        assert!(flat.contains("(stub)"), "missing stub marker: {flat}");
    }

    #[test]
    fn render_warn_delivery() {
        let backend = TestBackend::new(80, 32);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut runtime = sample_runtime();
        runtime.github_delivery_warn = Some("HMAC reject".into());
        runtime.last_github_delivery = Some(crate::status::GithubDeliverySnapshot {
            at_unix: 1,
            event: "push".into(),
            delivery_id: "d2".into(),
            outcome: "reject_hmac".into(),
            http_status: 401,
        });
        let model = StatusModel::new(
            "http://127.0.0.1:4700/health",
            "http://127.0.0.1:4700/status",
            HealthStatus::Ok,
            Some(runtime),
        );
        terminal.draw(|f| draw(f, &model)).expect("draw");
        let buffer = terminal.backend().buffer().clone();
        let flat: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(flat.contains("WARN"), "missing WARN: {flat}");
        assert!(flat.contains("HMAC reject"), "missing warn text: {flat}");
    }
}
