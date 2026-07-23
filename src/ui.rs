//! Ratatui widgets for the ITCy status pane.

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

#[derive(Debug, Clone)]
pub struct StatusModel {
    pub health_url: String,
    pub status_url: String,
    pub webhook_url: String,
    pub health: HealthStatus,
    pub runtime: Option<RuntimeStatus>,
    pub ticks: u64,
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
        }
    }
}

/// Same host as `/health`, path `POST /github/webhook_ITCy`.
fn hooks_url_from_health(health_url: &str) -> String {
    if health_url.ends_with("/health") {
        health_url.replacen("/health", "/github/webhook_ITCy", 1)
    } else {
        format!(
            "{}/github/webhook_ITCy",
            health_url.trim_end_matches('/')
        )
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
    draw_body(frame, chunks[1], model);
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
        Span::styled("Interchouette ITC status", Style::default().fg(Color::Yellow)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled("itcy-tui", ACCENT.add_modifier(Modifier::BOLD))),
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
    lines.push(Line::from(vec![
        Span::styled("keys:  ", LABEL),
        Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" / Esc / Ctrl-C  quit", MUTED),
        Span::styled("   ·   ", MUTED),
        Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("  refresh now", MUTED),
    ]));
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

    #[test]
    fn render_shows_ok_and_providers() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let runtime = RuntimeStatus {
            providers: vec!["ollama".into()],
            freeform_route_head: "ollama:gemma4:12b".into(),
            freeform_route: "ollama:gemma4:12b, ollama:llama3.1:8b".into(),
            draft_route_head: "ollama:gemma4:12b".into(),
            draft_route: "ollama:gemma4:12b".into(),
            github_webhook_configured: true,
            last_bat_wake: None,
        };
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
        assert!(flat.contains("ok"), "buffer missing ok: {flat}");
        assert!(flat.contains("providers"), "buffer missing providers");
        assert!(flat.contains("ollama"), "buffer missing ollama");
        assert!(flat.contains("webhook"), "buffer missing webhook");
        assert!(flat.contains("ok"), "buffer missing ok");
        assert!(
            flat.contains("/github/webhook_ITCy"),
            "buffer missing webhook url: {flat}"
        );
        assert!(flat.contains("ready"), "buffer missing webhook detail");
        assert!(
            !flat.contains("showcase"),
            "footer must not say showcase: {flat}"
        );
    }
}
