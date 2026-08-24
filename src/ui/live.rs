// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Live dashboard pane.

use super::model::StatusModel;
use super::style::{pane_block, ACCENT, LABEL, MUTED};
use crate::health::HealthStatus;
use crate::status::{EnrichStatusSnapshot, LinkedInMcpSnapshot, RuntimeStatus, TorListenSnapshot};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::Frame;

pub fn draw_live(frame: &mut Frame, area: Rect, model: &mut StatusModel) {
    model.hits.live_body = area;
    let mut lines = health_lines(model);
    lines.extend(runtime_lines(model));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("logs:  ", LABEL),
        Span::styled("product binary RUST_LOG stream - not this TUI", MUTED),
    ]));
    let n = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let mut scroll = ScrollbarState::new(usize::from(n)).position(usize::from(model.live_scroll));
    let body = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((model.live_scroll, 0))
        .block(pane_block("live", true));
    frame.render_widget(body, area);
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

fn labeled(label: &str, value: impl AsRef<str>, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<11}"), LABEL),
        Span::styled(value.as_ref().to_string(), value_style),
    ])
}

fn bold_status_line(label: &str, value: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<11}"), LABEL),
        Span::styled(
            value.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn health_lines(model: &StatusModel) -> Vec<Line<'static>> {
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
    let (ingress_color, ingress_detail) = match &model.ingress_health {
        HealthStatus::Ok => (Color::LightGreen, "ingress /health ok".to_string()),
        HealthStatus::Down { reason } => (Color::LightRed, reason.clone()),
    };
    vec![
        bold_status_line("health:", model.health.label(), health_color),
        labeled("url:", &model.health_url, ACCENT),
        labeled("detail:", &detail, detail_style),
        bold_status_line("ingress:", model.ingress_health.label(), ingress_color),
        labeled("url:", &model.ingress_url, ACCENT),
        labeled(
            "detail:",
            &ingress_detail,
            Style::default().fg(ingress_color),
        ),
        Line::from(""),
    ]
}

fn runtime_lines(model: &StatusModel) -> Vec<Line<'static>> {
    model.runtime.as_ref().map_or_else(
        || {
            vec![labeled(
                "routes:",
                format!("(unavailable - GET {})", model.status_url),
                Style::default().fg(Color::Yellow),
            )]
        },
        |rt| {
            let mut lines = route_lines(rt);
            lines.extend(webhook_lines(model, rt));
            lines.extend(delivery_lines(rt));
            lines.extend(enrich_lines(rt.enrich.as_ref()));
            lines.extend(tor_lines(rt.tor.as_ref()));
            lines.extend(linkedin_mcp_lines(rt.linkedin_mcp.as_ref()));
            lines
        },
    )
}

fn route_lines(rt: &RuntimeStatus) -> Vec<Line<'static>> {
    vec![
        labeled(
            "providers:",
            rt.providers_csv(),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        labeled(
            "freeform:",
            format!("head={} | {}", rt.freeform_route_head, rt.freeform_route),
            Style::default().fg(Color::LightBlue),
        ),
        labeled(
            "load:",
            format!("head={} | {}", rt.load_route_head, rt.load_route),
            Style::default().fg(Color::LightCyan),
        ),
        labeled(
            "draft:",
            format!("head={} | {}", rt.draft_route_head, rt.draft_route),
            Style::default().fg(Color::LightBlue),
        ),
        Line::from(""),
    ]
}

fn webhook_lines(model: &StatusModel, rt: &RuntimeStatus) -> Vec<Line<'static>> {
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
    vec![
        bold_status_line("webhook:", rt.webhook_label(), webhook_color),
        labeled("url:", &model.webhook_url, ACCENT),
        labeled("detail:", &webhook_detail, webhook_detail_style),
    ]
}

fn delivery_lines(rt: &RuntimeStatus) -> Vec<Line<'static>> {
    let delivery_label = rt.delivery_label();
    let delivery_color = match delivery_label {
        "ok" => Color::LightGreen,
        "WARN" => Color::LightRed,
        _ => Color::LightYellow,
    };
    let mut lines = vec![
        bold_status_line("delivery:", delivery_label, delivery_color),
        labeled(
            "last:",
            rt.delivery_detail(),
            Style::default().fg(Color::Gray),
        ),
    ];
    if let Some(warn) = rt.delivery_warn_line() {
        lines.push(labeled(
            "warn:",
            warn,
            Style::default().fg(Color::LightYellow),
        ));
    }
    lines.push(Line::from(""));
    lines
}

fn enrich_lines(enrich: Option<&EnrichStatusSnapshot>) -> Vec<Line<'static>> {
    enrich.map_or_else(
        || {
            vec![labeled(
                "enrich:",
                "(unavailable)",
                Style::default().fg(Color::Yellow),
            )]
        },
        |en| {
            let enrich_color = match en.enrich_label() {
                "running" => Color::LightGreen,
                "idle" => Color::Gray,
                _ => Color::LightYellow,
            };
            vec![
                bold_status_line("enrich:", en.enrich_label(), enrich_color),
                labeled(
                    "counts:",
                    en.enrich_detail(),
                    Style::default().fg(Color::LightCyan),
                ),
                labeled(
                    "queue:",
                    en.queue_detail(),
                    Style::default().fg(Color::LightBlue),
                ),
                labeled("wall:", en.wall_detail(), Style::default().fg(Color::Gray)),
            ]
        },
    )
}

fn tor_lines(tor: Option<&TorListenSnapshot>) -> Vec<Line<'static>> {
    tor.map_or_else(
        || {
            vec![labeled(
                "tor:",
                "(unavailable)",
                Style::default().fg(Color::Yellow),
            )]
        },
        |t| {
            let color = if t.ok {
                Color::LightGreen
            } else {
                Color::LightRed
            };
            let detail = if t.detail.is_empty() {
                if t.ok {
                    "ok".into()
                } else {
                    format!("socks={} control={}", t.socks_ok, t.control_ok)
                }
            } else {
                t.detail.clone()
            };
            vec![bold_status_line("tor:", &detail, color)]
        },
    )
}

fn linkedin_mcp_lines(mcp: Option<&LinkedInMcpSnapshot>) -> Vec<Line<'static>> {
    mcp.map_or_else(
        || {
            vec![labeled(
                "linkedin-mcp:",
                "(unavailable)",
                Style::default().fg(Color::Yellow),
            )]
        },
        |m| {
            let color = if m.ok {
                Color::LightGreen
            } else {
                Color::LightRed
            };
            let detail = if m.detail.is_empty() {
                if m.ok {
                    m.listen_addr.clone()
                } else {
                    "down".into()
                }
            } else if m.ok && !m.listen_addr.is_empty() {
                format!("{} ({})", m.detail, m.listen_addr)
            } else {
                m.detail.clone()
            };
            vec![bold_status_line("linkedin-mcp:", &detail, color)]
        },
    )
}
