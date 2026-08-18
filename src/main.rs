// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! `ITCy` ratatui status binary.

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, ExecutableCommand};
use itcy_tui::github::{PubsBranch, PubsRemote};
use itcy_tui::health::{
    fetch_health, replace_health_path, DEFAULT_HEALTH_URL, DEFAULT_INGRESS_HEALTH_URL,
};
use itcy_tui::status::{fetch_status, DEFAULT_STATUS_URL};
use itcy_tui::ui::{draw, StatusModel, ViewMode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::env;
use std::io::{self, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Set by SIGINT/SIGTERM so the loop can restore the tty before exit.
static STOP: AtomicBool = AtomicBool::new(false);

fn main() -> io::Result<()> {
    let health_url = env::var("ITCY_HEALTH_URL").unwrap_or_else(|_| DEFAULT_HEALTH_URL.to_string());
    let status_url = env::var("ITCY_STATUS_URL").unwrap_or_else(|_| {
        replace_health_path(&health_url, "/status")
            .unwrap_or_else(|| DEFAULT_STATUS_URL.to_string())
    });

    // Probe before taking over the terminal so HTTP noise never paints into the UI.
    let health = fetch_health(&health_url);
    let runtime = fetch_status(&status_url);

    let mut model = StatusModel::new(health_url.clone(), status_url.clone(), health, runtime);
    model.ingress_health = fetch_health(DEFAULT_INGRESS_HEALTH_URL);
    // GitHub tree fetch happens here (before raw mode) when landing on public pubs.
    model.apply_boot_view();

    install_signal_handlers();
    install_panic_hook();
    let use_alt = !inside_gnu_screen();
    let _guard = TerminalGuard::enter(use_alt)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    // Always hard-clear the surface we draw on (main buffer under GNU screen).
    terminal.clear()?;

    let poll = Duration::from_secs(1);
    let mut last = Instant::now().checked_sub(poll).unwrap();

    let result = run_loop(
        &mut terminal,
        &mut model,
        &health_url,
        &status_url,
        poll,
        &mut last,
    );

    // Wipe the drawn frame before Drop restores modes (avoids leftover boxes).
    let _ = terminal.clear();
    hard_reset_tty();
    result
}

fn refresh(model: &mut StatusModel, health_url: &str, status_url: &str) {
    model.health = fetch_health(health_url);
    model.ingress_health = fetch_health(DEFAULT_INGRESS_HEALTH_URL);
    model.runtime = fetch_status(status_url);
    model.ticks = model.ticks.saturating_add(1);
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    model: &mut StatusModel,
    health_url: &str,
    status_url: &str,
    poll: Duration,
    last: &mut Instant,
) -> io::Result<()> {
    loop {
        if STOP.load(Ordering::SeqCst) {
            break;
        }

        if last.elapsed() >= poll {
            refresh(model, health_url, status_url);
            *last = Instant::now();
        }

        terminal.draw(|frame| draw(frame, model))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match handle_key(model, key.code, key.modifiers) {
                    KeyAction::Quit => break,
                    KeyAction::RefreshProbes => {
                        refresh(model, health_url, status_url);
                        *last = Instant::now();
                    }
                    KeyAction::None => {}
                }
            }
        }
    }
    Ok(())
}

enum KeyAction {
    None,
    Quit,
    RefreshProbes,
}

fn handle_key(model: &mut StatusModel, code: KeyCode, modifiers: KeyModifiers) -> KeyAction {
    if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
        return KeyAction::Quit;
    }
    if model.view == ViewMode::Publications && model.pubs.filter_edit {
        return handle_filter_key(model, code);
    }
    match code {
        KeyCode::Char('q') | KeyCode::Esc => KeyAction::Quit,
        KeyCode::Char('r') => {
            if model.view == ViewMode::Publications {
                model.pubs.reload_tree();
            }
            if model.view == ViewMode::SavedList {
                model.show_saved_list();
            }
            KeyAction::RefreshProbes
        }
        KeyCode::Char('d') => {
            model.show_live();
            KeyAction::None
        }
        KeyCode::Char('c') => {
            model.show_commands();
            KeyAction::None
        }
        KeyCode::Char('p') => {
            model.show_publications();
            KeyAction::None
        }
        KeyCode::Char('s') => {
            model.show_saved_list();
            KeyAction::None
        }
        other if model.view == ViewMode::Publications => handle_pubs_nav(model, other),
        _ => KeyAction::None,
    }
}

fn handle_filter_key(model: &mut StatusModel, code: KeyCode) -> KeyAction {
    match code {
        KeyCode::Esc => model.pubs.end_filter(true),
        KeyCode::Enter => model.pubs.end_filter(false),
        KeyCode::Backspace => model.pubs.filter_pop(),
        KeyCode::Char(ch) => model.pubs.filter_push(ch),
        _ => {}
    }
    KeyAction::None
}

fn handle_pubs_nav(model: &mut StatusModel, code: KeyCode) -> KeyAction {
    match code {
        KeyCode::Char('j') | KeyCode::Down => model.pubs.move_selection(1),
        KeyCode::Char('k') | KeyCode::Up => model.pubs.move_selection(-1),
        KeyCode::Enter => model.pubs.load_selected(),
        KeyCode::Char('/') => model.pubs.begin_filter(),
        KeyCode::Char('o') => model.pubs.set_remote(PubsRemote::Org),
        KeyCode::Char('f') => model.pubs.set_remote(PubsRemote::Fork),
        KeyCode::Tab => model.pubs.cycle_branch(),
        KeyCode::Char(digit) => {
            if let Some(branch) = PubsBranch::from_digit(digit) {
                model.pubs.set_branch(branch);
            }
        }
        KeyCode::PageUp => model.pubs.scroll_body(-8),
        KeyCode::PageDown => model.pubs.scroll_body(8),
        _ => {}
    }
    KeyAction::None
}

/// GNU screen sets `STY`. Its default config often ignores the xterm alt buffer,
/// so Enter/LeaveAlternateScreen leaves a wrecked main buffer on exit.
fn inside_gnu_screen() -> bool {
    env::var_os("STY").is_some()
}

/// Reset SGR, cursor, and clear both alt and main tty buffers.
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
    if use_alt {
        let _ = out.execute(LeaveAlternateScreen);
    }
    let _ = out.flush();
    hard_reset_tty();
}

fn install_signal_handlers() {
    // Raw mode usually delivers Ctrl-C as a key; still restore if the process is signaled.
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

/// Restores the terminal on drop (normal exit, `?`, or unwind after panic hook).
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
