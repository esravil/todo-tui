use std::{
    io,
    path::Path,
    time::{Duration, Instant},
};
use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use todo_tui::{
    app::{input::handle_event, ui::draw, App},
    persistence,
};

fn main() -> Result<()> {
    let path = Path::new("todolist.json");

    // Run the TUI
    launch_tui(path)
}

fn launch_tui(path: &Path) -> Result<()> {
    let list = persistence::load(path)?;
    let mut app = App::new(list);

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let _guard = TerminalGuard;

    // tick config
    let tick_rate = Duration::from_millis(80);
    let mut last_tick = Instant::now();

    // Main loop
    loop {
        terminal.draw(|f| draw(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // poll with timeout so we also tick when there's no input
        if event::poll(timeout)? {
            let ev = event::read()?;
            if let Event::Key(_) | Event::Mouse(_) | Event::Resize(_, _) = ev {
                let keep = handle_event(&mut app, ev);
                if !keep {
                    if app.dirty {
                        persistence::save(path, &app.list)?;
                        app.dirty = false;
                    }
                    break;
                }
                if app.dirty {
                    persistence::save(path, &app.list)?;
                    app.dirty = false;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
    Ok(())
}

struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}