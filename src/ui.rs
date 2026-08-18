// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Ratatui widgets for the `ITCy` status pane.

use crate::artefact::{subject_from_meta, Artefact};
use crate::commands::SLASH_COMMANDS;
use crate::github::{fetch_branch_tree, fetch_file_text, PubsBranch, PubsRemote, TreeFetch};
use crate::health::{replace_health_path, HealthStatus, DEFAULT_INGRESS_HEALTH_URL};
use crate::inject::fetch_saved_list;
use crate::status::{EnrichStatusSnapshot, RuntimeStatus, TorListenSnapshot};
use std::collections::HashMap;

#[cfg(test)]
use crate::health::DEFAULT_HEALTH_URL;
#[cfg(test)]
use crate::status::DEFAULT_STATUS_URL;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

const LABEL: Style = Style::new().fg(Color::DarkGray);
const ACCENT: Style = Style::new().fg(Color::Cyan);
const MUTED: Style = Style::new().fg(Color::Gray);

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
    /// Localhost `/list` reply when the product is up.
    SavedList,
}

/// Pane model: probe URLs, last health/runtime snapshots, poll count, active view.
#[derive(Debug, Clone)]
pub struct StatusModel {
    /// Product `/health` URL.
    pub health_url: String,
    /// Product `/status` URL.
    pub status_url: String,
    /// Product webhook wake URL (`POST /hooks/github`).
    pub webhook_url: String,
    /// Ingress (`itc-hooks`) health URL.
    pub ingress_url: String,
    /// Last product health probe.
    pub health: HealthStatus,
    /// Last ingress health probe.
    pub ingress_health: HealthStatus,
    /// Last `/status` JSON, if fetch succeeded.
    pub runtime: Option<RuntimeStatus>,
    /// Poll counter for the live pane.
    pub ticks: u64,
    /// Active pane.
    pub view: ViewMode,
    /// Publications browser.
    pub pubs: PubsPane,
    /// `/list` reply text.
    pub saved_list: String,
    /// `/list` error when inject failed.
    pub saved_error: String,
}

/// GitHub publications browser state.
#[derive(Debug, Clone)]
pub struct PubsPane {
    /// Org or fork.
    pub remote: PubsRemote,
    /// Branch / kind.
    pub branch: PubsBranch,
    /// Cached artefacts for the current remote+branch.
    pub artefacts: Vec<Artefact>,
    /// Selected row among the filtered list.
    pub selected: usize,
    /// Id filter (substring).
    pub filter: String,
    /// True while typing a filter.
    pub filter_edit: bool,
    /// Loaded `body.md`.
    pub body: String,
    /// Subject from `meta.toml`.
    pub subject: String,
    /// Last tree or file error.
    pub error: String,
    /// GitHub rate-limit remaining.
    pub rate_remaining: Option<u32>,
    /// Preview vertical scroll.
    pub body_scroll: u16,
    /// Session cache: one tree per remote+branch.
    cache: HashMap<(PubsRemote, PubsBranch), TreeFetch>,
}

impl Default for PubsPane {
    fn default() -> Self {
        Self {
            remote: PubsRemote::Org,
            branch: PubsBranch::Drafts,
            artefacts: Vec::new(),
            selected: 0,
            filter: String::new(),
            filter_edit: false,
            body: String::new(),
            subject: String::new(),
            error: String::new(),
            rate_remaining: None,
            body_scroll: 0,
            cache: HashMap::new(),
        }
    }
}

impl PubsPane {
    fn apply_tree(&mut self, fetch: TreeFetch) {
        self.rate_remaining = fetch.rate_remaining;
        self.artefacts = fetch.artefacts;
        self.selected = 0;
        self.body.clear();
        self.subject.clear();
        self.body_scroll = 0;
        self.error = fetch.error.unwrap_or_default();
    }

    fn load_cached_or_fetch(&mut self) {
        let key = (self.remote, self.branch);
        if let Some(fetch) = self.cache.get(&key).cloned() {
            self.apply_tree(fetch);
            return;
        }
        self.reload_tree();
    }

    /// Fetch the current remote+branch tree from GitHub (replaces cache).
    pub fn reload_tree(&mut self) {
        let fetch = fetch_branch_tree(self.remote, self.branch);
        self.cache.insert((self.remote, self.branch), fetch.clone());
        self.apply_tree(fetch);
    }

    /// Switch org/fork and load the tree (cache first).
    pub fn set_remote(&mut self, remote: PubsRemote) {
        if self.remote == remote {
            return;
        }
        self.remote = remote;
        self.load_cached_or_fetch();
    }

    /// Cycle `drafts` → `posts` → `drafts_tweet` → `tweets`.
    pub fn cycle_branch(&mut self) {
        self.branch = self.branch.next();
        self.load_cached_or_fetch();
    }

    /// Jump to a publications branch.
    pub fn set_branch(&mut self, branch: PubsBranch) {
        if self.branch == branch {
            return;
        }
        self.branch = branch;
        self.load_cached_or_fetch();
    }

    /// Artefact indexes matching the filter.
    #[must_use]
    pub fn filtered_indexes(&self) -> Vec<usize> {
        let q = self.filter.to_ascii_lowercase();
        self.artefacts
            .iter()
            .enumerate()
            .filter(|(_, a)| q.is_empty() || a.id.to_ascii_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }

    /// Selected row in the filtered list.
    #[must_use]
    pub fn selected_artefact(&self) -> Option<&Artefact> {
        let idx = *self.filtered_indexes().get(self.selected)?;
        self.artefacts.get(idx)
    }

    fn clamp_selected(&mut self) {
        let n = self.filtered_indexes().len();
        if n == 0 {
            self.selected = 0;
        } else if self.selected >= n {
            self.selected = n.saturating_sub(1);
        }
    }

    /// Move selection in the filtered list.
    pub fn move_selection(&mut self, delta: i32) {
        let n = i32::try_from(self.filtered_indexes().len()).unwrap_or(0);
        if n == 0 {
            self.selected = 0;
            return;
        }
        let cur = i32::try_from(self.selected).unwrap_or(0);
        let next = (cur + delta).rem_euclid(n);
        self.selected = usize::try_from(next).unwrap_or(0);
    }

    /// Load body + meta for the selected artefact.
    pub fn load_selected(&mut self) {
        let Some((body_path, meta_path)) = self
            .selected_artefact()
            .map(|art| (art.body_path.clone(), art.meta_path.clone()))
        else {
            self.error = "no artefact selected".into();
            return;
        };
        match fetch_file_text(self.remote, self.branch, &body_path) {
            Ok(body) => {
                self.body = body;
                self.error.clear();
            }
            Err(e) => {
                self.body.clear();
                self.error = e;
            }
        }
        self.subject = fetch_file_text(self.remote, self.branch, &meta_path)
            .ok()
            .map(|m| subject_from_meta(&m))
            .unwrap_or_default();
        self.body_scroll = 0;
    }

    /// Begin substring filter edit.
    pub const fn begin_filter(&mut self) {
        self.filter_edit = true;
    }

    /// Append a filter character.
    pub fn filter_push(&mut self, ch: char) {
        self.filter.push(ch);
        self.selected = 0;
    }

    /// Delete the last filter character.
    pub fn filter_pop(&mut self) {
        self.filter.pop();
        self.clamp_selected();
    }

    /// Leave filter edit (`clear` wipes the query).
    pub fn end_filter(&mut self, clear: bool) {
        self.filter_edit = false;
        if clear {
            self.filter.clear();
            self.selected = 0;
        } else {
            self.clamp_selected();
        }
    }

    /// Scroll the `body.md` preview.
    pub fn scroll_body(&mut self, delta: i16) {
        if delta < 0 {
            self.body_scroll = self
                .body_scroll
                .saturating_sub(u16::try_from(-delta).unwrap_or(0));
        } else {
            self.body_scroll = self
                .body_scroll
                .saturating_add(u16::try_from(delta).unwrap_or(0));
        }
    }
}

impl StatusModel {
    /// Builds a model from health/status URLs and initial snapshots (`webhook_url` derived).
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
            ingress_url: DEFAULT_INGRESS_HEALTH_URL.to_string(),
            health,
            ingress_health: HealthStatus::Down {
                reason: "not probed".into(),
            },
            runtime,
            ticks: 0,
            view: ViewMode::Live,
            pubs: PubsPane::default(),
            saved_list: String::new(),
            saved_error: String::new(),
        }
    }

    /// Open the slash-command reference pane.
    pub const fn show_commands(&mut self) {
        self.view = ViewMode::Commands;
    }

    /// Open the live dashboard.
    pub const fn show_live(&mut self) {
        self.view = ViewMode::Live;
    }

    /// When product health is down, land on the publications browser.
    pub fn apply_boot_view(&mut self) {
        if !matches!(self.health, HealthStatus::Ok) {
            self.show_publications();
        }
    }

    /// Open the publications browser (fetch tree if empty).
    pub fn show_publications(&mut self) {
        self.view = ViewMode::Publications;
        if self.pubs.artefacts.is_empty() {
            self.pubs.reload_tree();
        }
    }

    /// Run localhost `/list` when health is ok.
    pub fn show_saved_list(&mut self) {
        self.view = ViewMode::SavedList;
        if !matches!(self.health, HealthStatus::Ok) {
            self.saved_list.clear();
            self.saved_error = "ITCy down - /list needs localhost :4700".into();
            return;
        }
        match fetch_saved_list(&self.health_url) {
            Ok(reply) => {
                self.saved_list = reply;
                self.saved_error.clear();
            }
            Err(e) => {
                self.saved_list.clear();
                self.saved_error = e;
            }
        }
    }

    /// Switch between live status and slash-command reference panes.
    pub const fn toggle_commands(&mut self) {
        self.view = match self.view {
            ViewMode::Commands => ViewMode::Live,
            _ => ViewMode::Commands,
        };
    }
}

/// Wake path on the product host (`POST /hooks/github`).
fn hooks_url_from_health(health_url: &str) -> String {
    replace_health_path(health_url, "/hooks/github")
        .unwrap_or_else(|| format!("{}/hooks/github", health_url.trim_end_matches('/')))
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

    draw_title(frame, chunks[0], model);
    match model.view {
        ViewMode::Live => draw_body(frame, chunks[1], model),
        ViewMode::Commands => draw_commands(frame, chunks[1], model),
        ViewMode::Publications => draw_publications(frame, chunks[1], model),
        ViewMode::SavedList => draw_saved_list(frame, chunks[1], model),
    }
    draw_footer(frame, chunks[2], model);
}

fn draw_title(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let subtitle = match model.view {
        ViewMode::Live => "Interchouette ITC status",
        ViewMode::Commands => "slash reference",
        ViewMode::Publications => "publications browser",
        ViewMode::SavedList => "saved list",
    };
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "ITCy",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ·  ", MUTED),
        Span::styled(subtitle, Style::default().fg(Color::Yellow)),
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
        HealthStatus::Ok => (Color::LightGreen, "itc-hooks /health ok".to_string()),
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
            lines
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

fn body_tail_lines() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        keys_line(),
        Line::from(vec![
            Span::styled("logs:  ", LABEL),
            Span::styled("product binary RUST_LOG stream - not this TUI", MUTED),
        ]),
    ]
}

fn draw_body(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let mut lines = health_lines(model);
    lines.extend(runtime_lines(model));
    lines.extend(body_tail_lines());

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

fn draw_commands(frame: &mut Frame, area: Rect, model: &StatusModel) {
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
        Line::from(vec![Span::styled(
            if matches!(model.health, HealthStatus::Ok) {
                "s  run /list via localhost POST /entrypoint/slash"
            } else {
                "s  /list needs ITCy :4700 (greyed: health down)"
            },
            if matches!(model.health, HealthStatus::Ok) {
                Style::default().fg(Color::LightGreen)
            } else {
                MUTED
            },
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

fn draw_publications(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);
    draw_pubs_list(frame, cols[0], model);
    draw_pubs_preview(frame, cols[1], model);
}

fn draw_pubs_list(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let pane = &model.pubs;
    let title = format!("pubs {} / {}", pane.remote.label(), pane.branch.label());
    let indexes = pane.filtered_indexes();
    let rows: Vec<Row> = indexes
        .iter()
        .enumerate()
        .map(|(vis, &idx)| {
            let art = &pane.artefacts[idx];
            let shard = art.shard.as_deref().unwrap_or("-");
            let style = if vis == pane.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(art.id.clone()),
                Cell::from(shard.to_string()),
            ])
            .style(style)
        })
        .collect();
    let table = Table::new(rows, [Constraint::Min(22), Constraint::Length(8)])
        .header(Row::new(vec!["id", "shard"]).style(LABEL.add_modifier(Modifier::BOLD)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                )),
        );
    frame.render_widget(table, area);
}

fn draw_pubs_preview(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let pane = &model.pubs;
    let id = pane
        .selected_artefact()
        .map_or_else(|| "(none)".into(), |a| a.id.clone());
    let mut lines = vec![
        labeled("id:", &id, ACCENT.add_modifier(Modifier::BOLD)),
        labeled(
            "subject:",
            if pane.subject.is_empty() {
                "(enter to load)"
            } else {
                pane.subject.as_str()
            },
            MUTED,
        ),
    ];
    if !pane.error.is_empty() {
        lines.push(labeled(
            "error:",
            &pane.error,
            Style::default().fg(Color::LightRed),
        ));
    }
    if pane.filter_edit {
        lines.push(labeled(
            "filter:",
            format!("{}█", pane.filter),
            Style::default().fg(Color::Yellow),
        ));
    } else if !pane.filter.is_empty() {
        lines.push(labeled("filter:", &pane.filter, MUTED));
    }
    lines.push(Line::from(""));
    for body_line in pane.body.lines() {
        lines.push(Line::from(Span::raw(body_line.to_string())));
    }
    if pane.body.is_empty() && pane.error.is_empty() {
        lines.push(Line::from(Span::styled(
            "Enter loads body.md  ·  o/f remote  ·  Tab kind  ·  / filter",
            MUTED,
        )));
    }
    let preview = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((pane.body_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(Span::styled(
                    "preview",
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )),
        );
    frame.render_widget(preview, area);
}

fn draw_saved_list(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let mut lines = Vec::new();
    if !model.saved_error.is_empty() {
        lines.push(Line::from(Span::styled(
            model.saved_error.clone(),
            Style::default().fg(Color::LightRed),
        )));
    } else if model.saved_list.is_empty() {
        lines.push(Line::from(Span::styled("(empty)", MUTED)));
    } else {
        for line in model.saved_list.lines() {
            lines.push(Line::from(Span::raw(line.to_string())));
        }
    }
    lines.push(Line::from(""));
    lines.push(keys_line());
    let body = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(Span::styled(
                "saved /list",
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
            "d",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  live", MUTED),
        Span::styled("   ·   ", MUTED),
        Span::styled(
            "c",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  commands", MUTED),
        Span::styled("   ·   ", MUTED),
        Span::styled(
            "p",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  pubs", MUTED),
        Span::styled("   ·   ", MUTED),
        Span::styled(
            "s",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  /list", MUTED),
    ])
}

const fn view_label(view: ViewMode) -> &'static str {
    match view {
        ViewMode::Live => "live",
        ViewMode::Commands => "commands",
        ViewMode::Publications => "pubs",
        ViewMode::SavedList => "list",
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, model: &StatusModel) {
    let mode = if matches!(model.health, HealthStatus::Ok) {
        "local"
    } else {
        "public"
    };
    let mut spans = vec![
        Span::styled("polls: ", LABEL),
        Span::styled(
            model.ticks.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   |   ", MUTED),
        Span::styled(mode, ACCENT.add_modifier(Modifier::BOLD)),
        Span::styled("   |   ", MUTED),
        Span::styled(view_label(model.view), ACCENT.add_modifier(Modifier::BOLD)),
    ];
    if model.view == ViewMode::Publications {
        if let Some(rate) = model.pubs.rate_remaining {
            spans.push(Span::styled("   |   rate ", MUTED));
            spans.push(Span::styled(
                rate.to_string(),
                Style::default().fg(Color::Yellow),
            ));
        }
        if let Some(art) = model.pubs.selected_artefact() {
            spans.push(Span::styled("   |   ", MUTED));
            spans.push(Span::styled(
                art.id.clone(),
                ACCENT.add_modifier(Modifier::BOLD),
            ));
        }
    }
    let footer = Paragraph::new(Line::from(spans));
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
            enrich: Some(crate::status::sample_enrich()),
            tor: Some(crate::status::TorListenSnapshot {
                ok: true,
                socks_ok: true,
                control_ok: true,
                detail: "ok".into(),
            }),
        }
    }

    #[test]
    fn render_shows_ok_and_providers() {
        let backend = TestBackend::new(80, 32);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
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
            flat.contains("/hooks/github"),
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
        let backend = TestBackend::new(100, 36);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut model = StatusModel::new(
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
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
            flat.contains("/ingest <url>"),
            "missing ingest usage: {flat}"
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
            DEFAULT_HEALTH_URL,
            DEFAULT_STATUS_URL,
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

    fn fixture_artefact(id: &str, shard: Option<&str>) -> Artefact {
        let folder = shard.map_or_else(|| id.to_string(), |s| format!("{s}/{id}"));
        Artefact {
            id: id.to_string(),
            shard: shard.map(ToString::to_string),
            body_path: format!("{folder}/body.md"),
            meta_path: format!("{folder}/meta.toml"),
        }
    }

    fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
        buffer
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect()
    }

    #[test]
    fn render_publications_fixture_tree() {
        let backend = TestBackend::new(100, 28);
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
            fixture_artefact("TWEET-20260813-000001", Some("2026/08")),
        ];
        model.pubs.body = "hello vitrine body".into();
        model.pubs.subject = "owl merge".into();
        terminal.draw(|f| draw(f, &model)).expect("draw");
        let flat = buffer_text(terminal.backend().buffer());
        assert!(
            flat.contains("DRAFT-20260801-000001"),
            "missing draft: {flat}"
        );
        assert!(
            flat.contains("TWEET-20260813-000001"),
            "missing tweet: {flat}"
        );
        assert!(flat.contains("2026/08"), "missing shard: {flat}");
        assert!(flat.contains("hello vitrine body"), "missing body: {flat}");
        assert!(flat.contains("owl merge"), "missing subject: {flat}");
        assert!(flat.contains("org"), "missing org tab: {flat}");
        assert!(flat.contains("drafts"), "missing branch: {flat}");
        assert!(flat.contains("public"), "missing public mode: {flat}");
        assert!(
            flat.contains("publications browser"),
            "missing title: {flat}"
        );
    }

    #[test]
    fn filter_narrows_indexes() {
        let pane = PubsPane {
            artefacts: vec![
                fixture_artefact("DRAFT-20260801-000001", None),
                fixture_artefact("TWEET-20260813-000001", Some("2026/08")),
            ],
            filter: "tweet".into(),
            ..PubsPane::default()
        };
        let idx = pane.filtered_indexes();
        assert_eq!(idx.len(), 1);
        assert_eq!(pane.artefacts[idx[0]].id, "TWEET-20260813-000001");
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
        model.apply_boot_view();
        assert_eq!(model.view, ViewMode::Publications);
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
        model.show_saved_list();
        assert_eq!(model.view, ViewMode::SavedList);
        terminal.draw(|f| draw(f, &model)).expect("draw");
        let flat = buffer_text(terminal.backend().buffer());
        assert!(flat.contains("/list needs"), "missing down message: {flat}");
    }
}
