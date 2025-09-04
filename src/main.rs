use std::{
    io,
    path::{Path},
    time::Duration,
};

use clap::Parser;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use todo_tui::{
    app::{self, input::handle_event, ui::draw, App},
    persistence,
    todolist::TodoList,
    Cmd, Cli,
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let data_path = cli.data_file.unwrap_or(persistence::default_path()?);

    match cli.cmd {
        Some(Cmd::Add { title, priority, notes }) => {
            let mut list = persistence::load(&data_path)?;
            list.add(&title, priority, notes);
            persistence::save(&data_path, &list)?;
            println!("Added: \"{}\" (priority {})", title, priority);
        }
        Some(Cmd::List) => {
            let list = persistence::load(&data_path)?;
            if list.items.is_empty() {
                println!("No tasks yet.");
            } else {
                for (i, t) in list.items.iter().enumerate() {
                    let state = if t.is_done() { "✓" } else { " " };
                    println!(
                        "{:>2}. [{}] [P{}] {}",
                        i,
                        state,
                        t.priority,
                        t.title
                    );
                }
            }
        }
        Some(Cmd::Done { index }) => {
            let mut list = persistence::load(&data_path)?;
            if list.toggle_done_index(index) {
                persistence::save(&data_path, &list)?;
                println!("Toggled task at index {}", index);
            } else {
                println!("No task at index {}", index);
            }
        }
        Some(Cmd::Delete { index }) => {
            let mut list = persistence::load(&data_path)?;
            if list.delete_index(index) {
                persistence::save(&data_path, &list)?;
                println!("Deleted task at index {}", index);
            } else {
                println!("No task at index {}", index);
            }
        }
        Some(Cmd::Tui) | None => {
            launch_tui(&data_path)?;
        }
    }

    Ok(())
}

fn launch_tui(path: &Path) -> Result<()> {
    // Load persisted state
    let list = persistence::load(path)?;
    let mut app = App::new(list);

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Ensure proper teardown even on panic
    let _guard = TerminalGuard;

    // Main loop
    loop {
        terminal.draw(|f| draw(f, &app))?;

        // Poll for events with a small timeout
        if event::poll(Duration::from_millis(250))? {
            let ev = event::read()?;
            if let Event::Key(_) | Event::Mouse(_) | Event::Resize(_, _) = ev {
                let keep_running = handle_event(&mut app, ev);
                if !keep_running {
                    // Save once before exiting if needed
                    if app.dirty {
                        persistence::save(path, &app.list)?;
                        app.dirty = false;
                    }
                    break;
                }
                if app.dirty {
                    persistence::save(path, &app.list)?;
                    app.dirty = false;
                    app.status_line = "Saved ✓".into();
                }
            }
        }
    }

    Ok(())
}

/// Restores the terminal on drop (even if we return early)
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Try best-effort cleanup; ignore errors at shutdown
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}
