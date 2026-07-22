//! Ratatui widgets for the ITCy status pane.

use crate::health::HealthStatus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

#[derive(Debug, Clone)]
pub struct StatusModel {
    pub health_url: String,
    pub health: HealthStatus,
    pub ticks: u64,
}

impl StatusModel {
    pub fn new(health_url: impl Into<String>, health: HealthStatus) -> Self {
        Self {
            health_url: health_url.into(),
            health,
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
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(frame.area());

    draw_title(frame, chunks[0]);
    draw_health(frame, chunks[1], model);
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

fn draw_health(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let (color, detail) = match &model.health {
        HealthStatus::Ok => (Color::Green, "product /health answered ok".to_string()),
        HealthStatus::Down { reason } => (Color::Red, reason.clone()),
    };

    let lines = vec![
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
        Line::from("q quit  ·  r refresh"),
    ];

    let body = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("live"));
    frame.render_widget(body, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let footer = Paragraph::new(format!(
        "polls: {}  ·  Written by AI - ITCy - ratatui showcase",
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
    fn render_shows_ok_label() {
        let backend = TestBackend::new(60, 16);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let model = StatusModel::new("http://127.0.0.1:4700/health", HealthStatus::Ok);
        terminal.draw(|f| draw(f, &model)).expect("draw");
        let buffer = terminal.backend().buffer().clone();
        let flat: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(flat.contains("ok"), "buffer missing ok: {flat}");
        assert!(flat.contains("ITCy"), "buffer missing brand");
    }

    #[test]
    fn render_shows_down_label() {
        let backend = TestBackend::new(60, 16);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let model = StatusModel::new(
            "http://127.0.0.1:4700/health",
            HealthStatus::Down {
                reason: "request: connection refused".into(),
            },
        );
        terminal.draw(|f| draw(f, &model)).expect("draw");
        let flat: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(flat.contains("DOWN"), "buffer missing DOWN: {flat}");
    }
}
