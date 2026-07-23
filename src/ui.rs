//! Ratatui widgets for the ITCy status pane.

use crate::health::HealthStatus;
use crate::status::RuntimeStatus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

#[derive(Debug, Clone)]
pub struct StatusModel {
    pub health_url: String,
    pub status_url: String,
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
        Self {
            health_url: health_url.into(),
            status_url: status_url.into(),
            health,
            runtime,
            ticks: 0,
        }
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
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  ·  Interchouette ITC status"),
    ]))
    .block(Block::default().borders(Borders::ALL).title("itcy-tui"));
    frame.render_widget(title, area);
}

fn draw_body(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let (color, detail) = match &model.health {
        HealthStatus::Ok => (Color::Green, "product /health answered ok".to_string()),
        HealthStatus::Down { reason } => (Color::Red, reason.clone()),
    };

    let mut lines = vec![
        Line::from(vec![
            Span::raw("health: "),
            Span::styled(
                model.health.label(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(format!("url:    {}", model.health_url)),
        Line::from(format!("detail: {detail}")),
        Line::from(""),
    ];

    match &model.runtime {
        Some(rt) => {
            lines.push(Line::from(format!("providers: {}", rt.providers_csv())));
            lines.push(Line::from(format!(
                "freeform:  head={} | {}",
                rt.freeform_route_head, rt.freeform_route
            )));
            lines.push(Line::from(format!(
                "draft:     head={} | {}",
                rt.draft_route_head, rt.draft_route
            )));
            let webhook_color = if rt.github_webhook_configured {
                Color::Green
            } else {
                Color::Yellow
            };
            lines.push(Line::from(vec![
                Span::raw("webhook:  "),
                Span::styled(
                    rt.webhook_label(),
                    Style::default()
                        .fg(webhook_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(format!("last wake: {}", rt.wake_summary())));
        }
        None => {
            lines.push(Line::from(format!(
                "routes:   (unavailable - GET {})",
                model.status_url
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(
        "keys:  q / Esc / Ctrl-C  quit   ·   r  refresh now",
    ));
    lines.push(Line::from(
        "logs:  product screen window `itcy` (RUST_LOG) - not this TUI",
    ));

    let body = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("live (full-screen TUI)"),
    );
    frame.render_widget(body, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let footer = Paragraph::new(format!(
        "polls: {}   |   Written by AI - ITCy - ratatui showcase",
        model.ticks
    ));
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
        assert!(flat.contains("configured"), "buffer missing configured");
        assert!(flat.contains("last wake"), "buffer missing last wake");
    }
}
