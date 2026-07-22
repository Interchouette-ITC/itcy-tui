//! ITCy ratatui status binary.

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use itcy_tui::health::{fetch_health, DEFAULT_HEALTH_URL};
use itcy_tui::ui::{draw, StatusModel};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::env;
use std::io::{self, stdout};
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    let health_url = env::var("ITCY_HEALTH_URL").unwrap_or_else(|_| DEFAULT_HEALTH_URL.to_string());

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut model = StatusModel::new(health_url.clone(), fetch_health(&health_url));
    let poll = Duration::from_secs(1);
    let mut last = Instant::now() - poll;

    let result = run_loop(&mut terminal, &mut model, &health_url, poll, &mut last);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    result
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
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('r') => {
                            model.health = fetch_health(health_url);
                            model.ticks = model.ticks.saturating_add(1);
                            *last = Instant::now();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
