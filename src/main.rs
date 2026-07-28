//! ITCy ratatui status binary.

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, ExecutableCommand};
use itcy_tui::health::{fetch_health, DEFAULT_HEALTH_URL};
use itcy_tui::status::{fetch_status, DEFAULT_STATUS_URL};
use itcy_tui::ui::{draw, StatusModel};
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
        if health_url.ends_with("/health") {
            health_url.replacen("/health", "/status", 1)
        } else {
            DEFAULT_STATUS_URL.to_string()
        }
    });

    // Probe before taking over the terminal so HTTP noise never paints into the UI.
    let health = fetch_health(&health_url);
    let runtime = fetch_status(&status_url);

    install_signal_handlers();
    install_panic_hook();
    let use_alt = !inside_gnu_screen();
    let _guard = TerminalGuard::enter(use_alt)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    // Always hard-clear the surface we draw on (main buffer under GNU screen).
    terminal.clear()?;

    let mut model = StatusModel::new(health_url.clone(), status_url.clone(), health, runtime);
    let poll = Duration::from_secs(1);
    let mut last = Instant::now() - poll;

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
                if key.kind == KeyEventKind::Press && is_quit_key(&key.code, key.modifiers) {
                    break;
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('r') {
                    refresh(model, health_url, status_url);
                    *last = Instant::now();
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('c') {
                    model.toggle_commands();
                }
            }
        }
    }
    Ok(())
}

fn is_quit_key(code: &KeyCode, modifiers: KeyModifiers) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Esc)
        || (*code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
}

/// GNU screen sets `STY`. Its default config often ignores the xterm alt buffer,
/// so Enter/LeaveAlternateScreen leaves a wrecked main buffer on exit.
fn inside_gnu_screen() -> bool {
    env::var_os("STY").is_some()
}

/// Nuclear tty reset: works on main buffer (screen) and after leaving alt screen.
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
