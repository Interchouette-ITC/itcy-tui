//! ITCy ratatui status binary.

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, ExecutableCommand};
use itcy_tui::health::{fetch_health, DEFAULT_HEALTH_URL};
use itcy_tui::ui::{draw, StatusModel};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::env;
use std::io::{self, stdout};
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    let health_url = env::var("ITCY_HEALTH_URL").unwrap_or_else(|_| DEFAULT_HEALTH_URL.to_string());

    install_panic_hook();
    let _guard = TerminalGuard::enter()?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut model = StatusModel::new(health_url.clone(), fetch_health(&health_url));
    let poll = Duration::from_secs(1);
    let mut last = Instant::now() - poll;

    run_loop(&mut terminal, &mut model, &health_url, poll, &mut last)
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    model: &mut StatusModel,
    health_url: &str,
    poll: Duration,
    last: &mut Instant,
) -> io::Result<()> {
    loop {
        if last.elapsed() >= poll {
            model.health = fetch_health(health_url);
            model.ticks = model.ticks.saturating_add(1);
            *last = Instant::now();
        }

        terminal.draw(|frame| draw(frame, model))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && is_quit_key(&key.code, key.modifiers) {
                    break;
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('r') {
                    model.health = fetch_health(health_url);
                    model.ticks = model.ticks.saturating_add(1);
                    *last = Instant::now();
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

fn restore_terminal() {
    let _ = disable_raw_mode();
    let mut out = stdout();
    let _ = out.execute(LeaveAlternateScreen);
    let _ = out.execute(cursor::Show);
}

fn install_panic_hook() {
    let prior = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        prior(info);
    }));
}

/// Restores the terminal on drop (normal exit, `?`, or unwind after panic hook).
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}
