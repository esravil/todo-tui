use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use super::{App, InputMode};

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

        KeyCode::Down => app.select_next(),
        KeyCode::Up => app.select_prev(),

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
                    app.status_line = "Toggled ✓".into();
                    app.dirty = true;
                }
            }
        }
        KeyCode::Char('d') => {
            if let Some(real_idx) = app.visible_indices().get(app.selected).cloned() {
                if app.list.delete_index(real_idx) {
                    app.status_line = "Deleted ✓".into();
                    app.dirty = true;
                    app.clamp_selection();
                }
            }
        }
        KeyCode::Char('s') => { app.status_line = "Saved ✓".into(); app.dirty = true; }

        // === NEW: tabs + visuals ==========================================
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => app.tabs.next(),
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => app.tabs.prev(),
        KeyCode::Char('t') => { app.show_chart = !app.show_chart; app.status_line = format!("Chart: {}", if app.show_chart { "On" } else { "Off" }); }
        KeyCode::Char('g') => { app.enhanced_graphics = !app.enhanced_graphics; app.status_line = format!("Graphics: {}", if app.enhanced_graphics { "Enhanced" } else { "Normal" }); }

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
                let notes_opt = {
                    let s = app.draft_notes.trim();
                    (!s.is_empty()).then(|| s.to_string())
                };
                app.list.add(title, app.draft_priority, notes_opt);
                app.status_line = "Added ✓".to_string();
                app.dirty = true;
                app.input_mode = InputMode::Normal;
            } else {
                app.status_line = "Title cannot be empty".into();
            }
        }
        KeyCode::Backspace => { app.draft_title.pop(); }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let n = c.to_digit(10).unwrap() as i8;
            if (1..=5).contains(&n) {
                app.draft_priority = n;
                app.status_line = format!("Priority set to {}", n);
            }
        }
        KeyCode::Char(c) => { app.draft_title.push(c); }
        _ => {}
    }
    true
}
