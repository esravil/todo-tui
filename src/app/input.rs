use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::{App, InputMode};

/// Handle one terminal event. Return `false` to exit the app.
pub fn handle_event(app: &mut App, ev: Event) -> bool {
    match ev {
        Event::Key(KeyEvent { code, modifiers, .. }) => match app.input_mode {
            InputMode::Normal => handle_normal_mode(app, code, modifiers),
            InputMode::Insert => handle_insert_mode(app, code, modifiers),
        },
        Event::Resize(_, _) => true,
        _ => true,
    }
}

fn handle_normal_mode(app: &mut App, code: KeyCode, _mods: KeyModifiers) -> bool {
    match code {
        KeyCode::Char('q') => return false,
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
        KeyCode::Char('a') => {
            app.input_mode = InputMode::Insert;
            app.draft_title.clear();
            app.draft_priority = 1;
            app.draft_notes.clear();
            app.status_line.clear();
        }
        KeyCode::Char('/') => app.cycle_filter(),
        KeyCode::Char(' ') => {
            if let Some(real_idx) = app.visible_indices().get(app.selected).cloned() {
                if app.list.toggle_done_index(real_idx) {
                    app.status_line = "Toggled ✓".to_string();
                    app.dirty = true;
                }
            }
        }
        KeyCode::Char('d') => {
            if let Some(real_idx) = app.visible_indices().get(app.selected).cloned() {
                if app.list.delete_index(real_idx) {
                    app.status_line = "Deleted ✓".to_string();
                    app.dirty = true;
                    app.clamp_selection();
                }
            }
        }
        KeyCode::Char('s') => {
            // Ask main loop to save: flipping dirty will trigger a save
            app.status_line = "Saved (requested) …".into();
            app.dirty = true;
        }
        _ => {}
    }
    true
}

fn handle_insert_mode(app: &mut App, code: KeyCode, _mods: KeyModifiers) -> bool {
    match code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.status_line = "Cancelled".into();
        }
        KeyCode::Enter => {
            let title = app.draft_title.trim();
            if !title.is_empty() {
                app.list
                    .add(title, app.draft_priority, some_if_nonempty(&app.draft_notes));
                app.status_line = "Added ✓".to_string();
                app.dirty = true;
                app.input_mode = InputMode::Normal;
            } else {
                app.status_line = "Title cannot be empty".into();
            }
        }
        KeyCode::Backspace => {
            app.draft_title.pop();
        }
        // Handle digit keys for priority (1–5)
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let n = c.to_digit(10).unwrap() as i8;
            if (1..=5).contains(&n) {
                app.draft_priority = n;
                app.status_line = format!("Priority set to {}", n);
            }
        }
        // Default: append to title
        KeyCode::Char(c) => {
            app.draft_title.push(c);
        }
        _ => {}
    }
    true
}

fn some_if_nonempty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
