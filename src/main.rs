// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! `ITCy` ratatui status binary.

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, ExecutableCommand};
use futures::StreamExt;
use itcy_tui::clipboard::copy_via_osc52;
use itcy_tui::github::{
    fetch_branch_tree, fetch_preview, PreviewFetch, PubsBranch, PubsRemote, TreeFetch,
};
use itcy_tui::health::{
    fetch_health, new_client, replace_health_path, HealthStatus, DEFAULT_HEALTH_URL,
    DEFAULT_INGRESS_HEALTH_URL,
};
use itcy_tui::inject::fetch_saved_list;
use itcy_tui::palette::{completions, parse_command, PaletteAction};
use itcy_tui::status::{fetch_status, DEFAULT_STATUS_URL};
use itcy_tui::ui::contains;
use itcy_tui::ui::{draw, InputMode, IoNeed, PubsFocus, StatusModel, ViewMode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::env;
use std::io::{self, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Set by SIGINT/SIGTERM so the loop can restore the tty before exit.
static STOP: AtomicBool = AtomicBool::new(false);

struct IoHub<'a> {
    client: &'a reqwest::Client,
    health_url: &'a str,
    status_url: &'a str,
    tx: &'a mpsc::UnboundedSender<IoEvent>,
    preview_job: &'a mut Option<tokio::task::JoinHandle<()>>,
    tree_busy: &'a mut bool,
    saved_busy: &'a mut bool,
}

enum IoEvent {
    Tree(TreeFetch),
    Preview { gen: u64, fetch: PreviewFetch },
    Saved(Result<String, String>),
    Probes(Box<ProbeSnap>),
}

struct ProbeSnap {
    health: HealthStatus,
    ingress: HealthStatus,
    runtime: Option<itcy_tui::RuntimeStatus>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = new_client().map_err(io::Error::other)?;
    let health_url = env::var("ITCY_HEALTH_URL").unwrap_or_else(|_| DEFAULT_HEALTH_URL.to_string());
    let status_url = env::var("ITCY_STATUS_URL").unwrap_or_else(|_| {
        replace_health_path(&health_url, "/status")
            .unwrap_or_else(|| DEFAULT_STATUS_URL.to_string())
    });

    let health = fetch_health(&client, &health_url).await;
    let runtime = fetch_status(&client, &status_url).await;
    let mut model = StatusModel::new(health_url.clone(), status_url.clone(), health, runtime);
    model.ingress_health = fetch_health(&client, DEFAULT_INGRESS_HEALTH_URL).await;
    if model.apply_boot_view() == IoNeed::Tree {
        let fetch = fetch_branch_tree(&client, model.pubs.remote, model.pubs.branch).await;
        model.pubs.apply_tree(fetch);
    }

    install_signal_handlers();
    install_panic_hook();
    let use_alt = !inside_gnu_screen();
    let _guard = TerminalGuard::enter(use_alt)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_loop(&mut terminal, &mut model, &client, &health_url, &status_url).await;
    let _ = terminal.clear();
    hard_reset_tty();
    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    model: &mut StatusModel,
    client: &reqwest::Client,
    health_url: &str,
    status_url: &str,
) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<IoEvent>();
    let mut events = EventStream::new();
    let mut last_probe = Instant::now();
    let mut preview_job: Option<tokio::task::JoinHandle<()>> = None;
    let mut tree_busy = false;
    let mut saved_busy = false;
    loop {
        if STOP.load(Ordering::SeqCst) {
            break;
        }
        terminal.draw(|frame| draw(frame, model))?;
        tokio::select! {
            maybe = events.next() => {
                let Some(Ok(ev)) = maybe else { continue };
                match ev {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        match handle_key(model, key) {
                            LoopAction::Quit => break,
                            LoopAction::Yank => yank(model),
                            LoopAction::Io(need) => spawn_io(
                                need,
                                model,
                                &mut IoHub {
                                    client,
                                    health_url,
                                    status_url,
                                    tx: &tx,
                                    preview_job: &mut preview_job,
                                    tree_busy: &mut tree_busy,
                                    saved_busy: &mut saved_busy,
                                },
                            ),
                            LoopAction::None => {}
                        }
                    }
                    Event::Mouse(mouse) => {
                        match handle_mouse(model, mouse) {
                            LoopAction::Quit => break,
                            LoopAction::Yank => yank(model),
                            LoopAction::Io(need) => spawn_io(
                                need,
                                model,
                                &mut IoHub {
                                    client,
                                    health_url,
                                    status_url,
                                    tx: &tx,
                                    preview_job: &mut preview_job,
                                    tree_busy: &mut tree_busy,
                                    saved_busy: &mut saved_busy,
                                },
                            ),
                            LoopAction::None => {}
                        }
                    }
                    _ => {}
                }
            }
            Some(io) = rx.recv() => {
                let is_tree = matches!(io, IoEvent::Tree(_));
                let is_saved = matches!(io, IoEvent::Saved(_));
                apply_io(model, io);
                if is_tree {
                    tree_busy = false;
                }
                if is_saved {
                    saved_busy = false;
                }
                model.pending = preview_job.as_ref().is_some_and(|h| !h.is_finished())
                    || tree_busy
                    || saved_busy;
            }
            () = tokio::time::sleep(Duration::from_millis(200)) => {
                on_tick(
                    model,
                    &mut IoHub {
                        client,
                        health_url,
                        status_url,
                        tx: &tx,
                        preview_job: &mut preview_job,
                        tree_busy: &mut tree_busy,
                        saved_busy: &mut saved_busy,
                    },
                    &mut last_probe,
                );
            }
        }
    }
    Ok(())
}

enum LoopAction {
    None,
    Quit,
    Yank,
    Io(IoNeed),
}

fn on_tick(model: &mut StatusModel, hub: &mut IoHub<'_>, last_probe: &mut Instant) {
    model.ticks = model.ticks.saturating_add(1);
    if last_probe.elapsed() >= Duration::from_secs(1) {
        *last_probe = Instant::now();
        spawn_probes(hub.client, hub.health_url, hub.status_url, hub.tx);
    }
    if model.view == ViewMode::Publications
        && model.pubs.preview_dirty
        && model
            .pubs
            .last_select
            .is_some_and(|t| t.elapsed() >= Duration::from_millis(400))
    {
        spawn_io(IoNeed::Preview, model, hub);
    }
}

fn spawn_probes(
    client: &reqwest::Client,
    health_url: &str,
    status_url: &str,
    tx: &mpsc::UnboundedSender<IoEvent>,
) {
    let client = client.clone();
    let health_url = health_url.to_string();
    let status_url = status_url.to_string();
    let tx = tx.clone();
    tokio::spawn(async move {
        let health = fetch_health(&client, &health_url).await;
        let ingress = fetch_health(&client, DEFAULT_INGRESS_HEALTH_URL).await;
        let runtime = fetch_status(&client, &status_url).await;
        let _ = tx.send(IoEvent::Probes(Box::new(ProbeSnap {
            health,
            ingress,
            runtime,
        })));
    });
}

fn spawn_io(need: IoNeed, model: &mut StatusModel, hub: &mut IoHub<'_>) {
    match need {
        IoNeed::None => {}
        IoNeed::Tree => {
            if *hub.tree_busy {
                return;
            }
            *hub.tree_busy = true;
            model.pending = true;
            let client = hub.client.clone();
            let remote = model.pubs.remote;
            let branch = model.pubs.branch;
            let tx = hub.tx.clone();
            tokio::spawn(async move {
                let fetch = fetch_branch_tree(&client, remote, branch).await;
                let _ = tx.send(IoEvent::Tree(fetch));
            });
        }
        IoNeed::Preview => {
            let Some(art) = model.pubs.selected_artefact() else {
                return;
            };
            let gen = model.pubs.select_gen;
            let id = art.id.clone();
            let body_path = art.body_path.clone();
            let meta_path = art.meta_path.clone();
            let remote = model.pubs.remote;
            let branch = model.pubs.branch;
            model.pubs.preview_dirty = false;
            if let Some(h) = hub.preview_job.take() {
                h.abort();
            }
            model.pending = true;
            let client = hub.client.clone();
            let tx = hub.tx.clone();
            *hub.preview_job = Some(tokio::spawn(async move {
                let fetch = fetch_preview(&client, remote, branch, id, body_path, meta_path).await;
                let _ = tx.send(IoEvent::Preview { gen, fetch });
            }));
        }
        IoNeed::SavedList => {
            if *hub.saved_busy {
                return;
            }
            *hub.saved_busy = true;
            model.pending = true;
            let client = hub.client.clone();
            let health_url = hub.health_url.to_string();
            let tx = hub.tx.clone();
            tokio::spawn(async move {
                let reply = fetch_saved_list(&client, &health_url).await;
                let _ = tx.send(IoEvent::Saved(reply));
            });
        }
    }
}

fn apply_io(model: &mut StatusModel, io: IoEvent) {
    match io {
        IoEvent::Tree(fetch) => {
            model.pubs.apply_tree(fetch);
            model.pending = false;
        }
        IoEvent::Preview { gen, fetch } => {
            model
                .pubs
                .apply_preview(gen, &fetch.id, fetch.body, fetch.subject);
            model.pending = false;
        }
        IoEvent::Saved(reply) => {
            model.apply_saved_reply(reply);
            model.pending = false;
        }
        IoEvent::Probes(snap) => {
            model.health = snap.health;
            model.ingress_health = snap.ingress;
            model.runtime = snap.runtime;
        }
    }
}

fn yank(model: &mut StatusModel) {
    if let Some(text) = model.yank_text() {
        if copy_via_osc52(&text).is_ok() {
            model.copied = true;
            model.notice.clear();
        } else {
            model.notice = "copy failed".into();
        }
    }
}

fn handle_key(model: &mut StatusModel, key: KeyEvent) -> LoopAction {
    model.copied = false;
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return LoopAction::Quit;
    }
    if model.input != InputMode::Normal {
        return handle_overlay(model, key);
    }
    match key.code {
        KeyCode::Char('q') => LoopAction::Quit,
        KeyCode::Esc => {
            model.leave_help();
            LoopAction::None
        }
        KeyCode::Char('?') => {
            if model.view == ViewMode::Help {
                model.leave_help();
            } else {
                model.show_help();
            }
            LoopAction::None
        }
        KeyCode::Char('d') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            model.show_live();
            LoopAction::None
        }
        KeyCode::Char('c') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            model.show_commands();
            LoopAction::None
        }
        KeyCode::Char('p') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            LoopAction::Io(model.show_publications())
        }
        KeyCode::Char('s') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            LoopAction::Io(model.show_saved_list())
        }
        KeyCode::Char('/') => {
            if matches!(
                model.view,
                ViewMode::Publications | ViewMode::Commands | ViewMode::SavedList
            ) {
                model.begin_filter();
            }
            LoopAction::None
        }
        KeyCode::Char(':') => {
            model.begin_command();
            LoopAction::None
        }
        KeyCode::Char('y') => LoopAction::Yank,
        KeyCode::Char('r') => refresh_action(model),
        other => handle_view_key(model, other, key.modifiers),
    }
}

fn refresh_action(model: &mut StatusModel) -> LoopAction {
    match model.view {
        ViewMode::Publications => LoopAction::Io(IoNeed::Tree),
        ViewMode::SavedList => LoopAction::Io(model.show_saved_list()),
        _ => LoopAction::None,
    }
}

fn handle_overlay(model: &mut StatusModel, key: KeyEvent) -> LoopAction {
    match key.code {
        KeyCode::Esc => {
            model.end_overlay(model.input == InputMode::Filter);
            LoopAction::None
        }
        KeyCode::Enter => commit_overlay(model),
        KeyCode::Tab if model.input == InputMode::Command => {
            tab_complete(model);
            LoopAction::None
        }
        KeyCode::Up if model.input == InputMode::Command => {
            if let Some(line) = model.history.up() {
                let line = line.to_string();
                set_textarea(model, &line);
            }
            LoopAction::None
        }
        KeyCode::Down if model.input == InputMode::Command => {
            let line = model.history.down().unwrap_or("").to_string();
            set_textarea(model, &line);
            LoopAction::None
        }
        _ => {
            model.textarea.input(key);
            if model.input == InputMode::Filter {
                model.sync_filter_from_overlay();
            }
            LoopAction::None
        }
    }
}

fn set_textarea(model: &mut StatusModel, line: &str) {
    model.textarea.set_line(line);
}

fn tab_complete(model: &mut StatusModel) {
    let buf = model.overlay_line();
    if model.completions.is_empty() {
        model.completions = completions(&buf);
        model.completion_idx = None;
    }
    if model.completions.is_empty() {
        return;
    }
    let next = match model.completion_idx {
        None => 0,
        Some(i) => (i + 1) % model.completions.len(),
    };
    model.completion_idx = Some(next);
    let chosen = model.completions[next].clone();
    set_textarea(model, &chosen);
}

fn commit_overlay(model: &mut StatusModel) -> LoopAction {
    if model.input == InputMode::Filter {
        model.sync_filter_from_overlay();
        model.end_overlay(false);
        return LoopAction::None;
    }
    let line = model.overlay_line();
    model.history.push(&line);
    model.end_overlay(false);
    match parse_command(&line) {
        PaletteAction::View(ViewMode::Live) => {
            model.show_live();
            LoopAction::None
        }
        PaletteAction::View(ViewMode::Commands) => {
            model.show_commands();
            LoopAction::None
        }
        PaletteAction::View(ViewMode::Publications) => LoopAction::Io(model.show_publications()),
        PaletteAction::View(ViewMode::SavedList) => LoopAction::Io(model.show_saved_list()),
        PaletteAction::View(ViewMode::Help) => {
            model.show_help();
            LoopAction::None
        }
        PaletteAction::Remote(r) => LoopAction::Io(model.pubs.set_remote(r)).and_pubs(model),
        PaletteAction::Branch(b) => LoopAction::Io(model.pubs.set_branch(b)).and_pubs(model),
        PaletteAction::Reload => {
            model.view = ViewMode::Publications;
            LoopAction::Io(IoNeed::Tree)
        }
        PaletteAction::Open(prefix) => {
            model.view = ViewMode::Publications;
            if model.pubs.open_prefix(&prefix) {
                LoopAction::Io(IoNeed::Preview)
            } else {
                model.notice = format!("no match for {prefix}");
                LoopAction::None
            }
        }
        PaletteAction::Unknown(s) => {
            model.notice = format!("unknown :{s}");
            LoopAction::None
        }
    }
}

impl LoopAction {
    const fn and_pubs(self, model: &mut StatusModel) -> Self {
        model.view = ViewMode::Publications;
        self
    }
}

fn handle_view_key(model: &mut StatusModel, code: KeyCode, modifiers: KeyModifiers) -> LoopAction {
    match model.view {
        ViewMode::Publications => handle_pubs_key(model, code, modifiers),
        ViewMode::Commands => handle_cmd_key(model, code),
        ViewMode::SavedList => handle_saved_key(model, code),
        ViewMode::Live => {
            match code {
                KeyCode::PageDown | KeyCode::Down => {
                    model.live_scroll = model.live_scroll.saturating_add(4);
                }
                KeyCode::PageUp | KeyCode::Up => {
                    model.live_scroll = model.live_scroll.saturating_sub(4);
                }
                _ => {}
            }
            LoopAction::None
        }
        ViewMode::Help => {
            match code {
                KeyCode::PageDown | KeyCode::Down | KeyCode::Char('j') => {
                    model.help_scroll = model.help_scroll.saturating_add(2);
                }
                KeyCode::PageUp | KeyCode::Up | KeyCode::Char('k') => {
                    model.help_scroll = model.help_scroll.saturating_sub(2);
                }
                _ => {}
            }
            LoopAction::None
        }
    }
}

fn handle_pubs_key(model: &mut StatusModel, code: KeyCode, modifiers: KeyModifiers) -> LoopAction {
    if modifiers.contains(KeyModifiers::CONTROL) {
        match code {
            KeyCode::Char('d') => model.pubs.page(true),
            KeyCode::Char('u') => model.pubs.page(false),
            _ => {}
        }
        return LoopAction::None;
    }
    match code {
        KeyCode::Char('j') | KeyCode::Down if model.pubs.focus == PubsFocus::List => {
            model.pubs.move_selection(1);
            LoopAction::None
        }
        KeyCode::Char('k') | KeyCode::Up if model.pubs.focus == PubsFocus::List => {
            model.pubs.move_selection(-1);
            LoopAction::None
        }
        KeyCode::Char('j') | KeyCode::Down => {
            model.pubs.scroll_body(4);
            LoopAction::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            model.pubs.scroll_body(-4);
            LoopAction::None
        }
        KeyCode::Char('g') | KeyCode::Home => {
            model.pubs.select_edge(false);
            LoopAction::None
        }
        KeyCode::Char('G') | KeyCode::End => {
            model.pubs.select_edge(true);
            LoopAction::None
        }
        KeyCode::Enter => LoopAction::Io(IoNeed::Preview),
        KeyCode::Tab => {
            model.pubs.focus = match model.pubs.focus {
                PubsFocus::List => PubsFocus::Preview,
                PubsFocus::Preview => PubsFocus::List,
            };
            LoopAction::None
        }
        KeyCode::BackTab => {
            model.pubs.focus = PubsFocus::List;
            LoopAction::None
        }
        KeyCode::Char('o') => LoopAction::Io(model.pubs.set_remote(PubsRemote::Org)),
        KeyCode::Char('f') => LoopAction::Io(model.pubs.set_remote(PubsRemote::Fork)),
        KeyCode::Char(d) => {
            if let Some(b) = PubsBranch::from_digit(d) {
                LoopAction::Io(model.pubs.set_branch(b))
            } else {
                LoopAction::None
            }
        }
        KeyCode::PageUp => {
            model.pubs.scroll_body(-8);
            LoopAction::None
        }
        KeyCode::PageDown => {
            model.pubs.scroll_body(8);
            LoopAction::None
        }
        _ => LoopAction::None,
    }
}

fn handle_cmd_key(model: &mut StatusModel, code: KeyCode) -> LoopAction {
    match code {
        KeyCode::Char('j') | KeyCode::Down => model.move_cmd(1),
        KeyCode::Char('k') | KeyCode::Up => model.move_cmd(-1),
        KeyCode::Char('g') | KeyCode::Home => model.cmd_state.select(Some(0)),
        KeyCode::Char('G') | KeyCode::End => {
            let n = model.filtered_commands().len();
            if n > 0 {
                model.cmd_state.select(Some(n - 1));
            }
        }
        KeyCode::Enter if model.selected_is_list_cmd() => {
            return LoopAction::Io(model.show_saved_list());
        }
        _ => {}
    }
    LoopAction::None
}

fn handle_saved_key(model: &mut StatusModel, code: KeyCode) -> LoopAction {
    match code {
        KeyCode::Char('j') | KeyCode::Down => model.move_saved(1),
        KeyCode::Char('k') | KeyCode::Up => model.move_saved(-1),
        KeyCode::Char('g') | KeyCode::Home => model.saved_state.select(Some(0)),
        KeyCode::Char('G') | KeyCode::End => {
            let n = model.filtered_saved().len();
            if n > 0 {
                model.saved_state.select(Some(n - 1));
            }
        }
        _ => {}
    }
    LoopAction::None
}

fn handle_mouse(model: &mut StatusModel, mouse: MouseEvent) -> LoopAction {
    model.copied = false;
    let col = mouse.column;
    let row = mouse.row;
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => click(model, col, row),
        MouseEventKind::ScrollDown => wheel(model, col, row, 3),
        MouseEventKind::ScrollUp => wheel(model, col, row, -3),
        _ => LoopAction::None,
    }
}

fn click(model: &mut StatusModel, col: u16, row: u16) -> LoopAction {
    let h = model.hits.clone();
    if contains(h.live_tab, col, row) {
        model.show_live();
        return LoopAction::None;
    }
    if contains(h.commands_tab, col, row) {
        model.show_commands();
        return LoopAction::None;
    }
    if contains(h.pubs_tab, col, row) {
        return LoopAction::Io(model.show_publications());
    }
    if contains(h.list_tab, col, row) {
        return LoopAction::Io(model.show_saved_list());
    }
    if model.view == ViewMode::Publications {
        if contains(h.org_chip, col, row) {
            return LoopAction::Io(model.pubs.set_remote(PubsRemote::Org));
        }
        if contains(h.fork_chip, col, row) {
            return LoopAction::Io(model.pubs.set_remote(PubsRemote::Fork));
        }
        for (i, chip) in h.branch_chips.iter().enumerate() {
            if contains(*chip, col, row) {
                if let Some(b) = PubsBranch::ALL.get(i).copied() {
                    return LoopAction::Io(model.pubs.set_branch(b));
                }
            }
        }
        if contains(h.pubs_list, col, row) {
            model.pubs.focus = PubsFocus::List;
            let inner_y = row.saturating_sub(h.pubs_list.y).saturating_sub(2);
            model.pubs.select_visible(usize::from(inner_y));
            return LoopAction::None;
        }
        if contains(h.pubs_preview, col, row) {
            model.pubs.focus = PubsFocus::Preview;
        }
    }
    if model.view == ViewMode::Commands && contains(h.commands_table, col, row) {
        select_table_row(
            h.commands_table,
            col,
            row,
            model.filtered_commands().len(),
            |i| {
                model.cmd_state.select(Some(i));
            },
        );
    }
    if model.view == ViewMode::SavedList && contains(h.saved_table, col, row) {
        select_table_row(h.saved_table, col, row, model.filtered_saved().len(), |i| {
            model.saved_state.select(Some(i));
        });
    }
    LoopAction::None
}

fn select_table_row(
    area: ratatui::layout::Rect,
    _col: u16,
    row: u16,
    n: usize,
    mut set: impl FnMut(usize),
) {
    let inner_y = row.saturating_sub(area.y).saturating_sub(2);
    let idx = usize::from(inner_y);
    if idx < n {
        set(idx);
    }
}

fn wheel(model: &mut StatusModel, col: u16, row: u16, delta: i16) -> LoopAction {
    let h = model.hits.clone();
    if model.view == ViewMode::Publications && contains(h.pubs_list, col, row) {
        model.pubs.move_selection(i32::from(delta.signum()));
        return LoopAction::None;
    }
    if model.view == ViewMode::Publications && contains(h.pubs_preview, col, row) {
        model.pubs.scroll_body(delta);
        return LoopAction::None;
    }
    if model.view == ViewMode::Live && contains(h.live_body, col, row) {
        if delta > 0 {
            model.live_scroll = model
                .live_scroll
                .saturating_add(u16::try_from(delta).unwrap_or(0));
        } else {
            model.live_scroll = model
                .live_scroll
                .saturating_sub(u16::try_from(-delta).unwrap_or(0));
        }
    }
    if model.view == ViewMode::Help && contains(h.help_body, col, row) {
        if delta > 0 {
            model.help_scroll = model
                .help_scroll
                .saturating_add(u16::try_from(delta).unwrap_or(0));
        } else {
            model.help_scroll = model
                .help_scroll
                .saturating_sub(u16::try_from(-delta).unwrap_or(0));
        }
    }
    if model.view == ViewMode::Commands && contains(h.commands_table, col, row) {
        model.move_cmd(i32::from(delta.signum()));
    }
    if model.view == ViewMode::SavedList && contains(h.saved_table, col, row) {
        model.move_saved(i32::from(delta.signum()));
    }
    LoopAction::None
}

fn inside_gnu_screen() -> bool {
    env::var_os("STY").is_some()
}

fn hard_reset_tty() {
    let mut out = stdout();
    let _ = write!(
        out,
        "\x1b[0m\x1b[?25h\x1b[?1049l\x1b[?47l\x1b[2J\x1b[3J\x1b[H"
    );
    let _ = out.execute(Clear(ClearType::All));
    let _ = out.execute(Clear(ClearType::Purge));
    let _ = out.execute(cursor::Show);
    let _ = out.flush();
}

fn restore_terminal(use_alt: bool) {
    let _ = disable_raw_mode();
    let mut out = stdout();
    let _ = out.execute(DisableMouseCapture);
    if use_alt {
        let _ = out.execute(LeaveAlternateScreen);
    }
    let _ = out.flush();
    hard_reset_tty();
}

fn install_signal_handlers() {
    let _ = ctrlc::set_handler(|| {
        STOP.store(true, Ordering::SeqCst);
    });
}

fn install_panic_hook() {
    let use_alt = !inside_gnu_screen();
    let prior = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal(use_alt);
        prior(info);
    }));
}

struct TerminalGuard {
    use_alt: bool,
}

impl TerminalGuard {
    fn enter(use_alt: bool) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        if use_alt {
            out.execute(EnterAlternateScreen)?;
        }
        out.execute(EnableMouseCapture)?;
        hard_reset_tty();
        out.execute(cursor::Hide)?;
        out.flush()?;
        Ok(Self { use_alt })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal(self.use_alt);
    }
}
