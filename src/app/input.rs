use crossterm::event::{Event, KeyCode, KeyEvent};
use crate::app::{App, InputMode, InsertField, MapView};

pub fn handle_event(app: &mut App, ev: Event) -> bool {
    match ev {
        Event::Key(KeyEvent { code, .. }) => match app.input_mode {
            InputMode::Normal => handle_normal_mode(app, code),
            InputMode::Insert => handle_insert_mode(app, code),
        },
        Event::Resize(_, _) => true,
        _ => true,
    }
}

fn handle_normal_mode(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Char('q') => return false,

        KeyCode::Down => app.select_next(),
        KeyCode::Up => app.select_prev(),

        KeyCode::Char('a') => {
            app.input_mode = InputMode::Insert;
            app.insert_field = InsertField::Title;
            app.draft_title.clear();
            app.draft_priority = 1;
            app.draft_notes.clear();
            app.draft_timeframe.clear();
            app.status_line.clear();
        }

        // Expand/collapse inline details with Space
        KeyCode::Char(' ') => {
            app.expanded = !app.expanded;
            app.status_line = if app.expanded { "Expanded".into() } else { "Collapsed".into() };
        }

        // Toggle done on Enter
        KeyCode::Enter => {
            if let Some(real_idx) = app.visible_indices().get(app.selected).cloned() {
                if app.list.toggle_done_index(real_idx) {
                    app.status_line = "Toggled ✓".into();
                    app.dirty = true;
                }
            }
        }

        // Delete
        KeyCode::Char('d') => {
            if let Some(real_idx) = app.visible_indices().get(app.selected).cloned() {
                if app.list.delete_index(real_idx) {
                    app.status_line = "Deleted ✓".into();
                    app.dirty = true;
                    app.clamp_selection();
                }
            }
        }

        // Save marker
        KeyCode::Char('s') => { app.status_line = "Saved ✓".into(); app.dirty = true; }

        // Map view toggle (World <-> NYC)
        KeyCode::Char('m') => {
            app.map_view = match app.map_view {
                MapView::World => MapView::NYC,
                MapView::NYC => MapView::World,
            };
            app.status_line = format!("Map view: {}", match app.map_view { MapView::World => "World", MapView::NYC => "NYC" });
        }

        // tabs + visuals
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => app.tabs.next(),
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => app.tabs.prev(),
        KeyCode::Char('t') => { app.show_chart = !app.show_chart; app.status_line = format!("Chart: {}", if app.show_chart { "On" } else { "Off" }); }
        KeyCode::Char('g') => { app.enhanced_graphics = !app.enhanced_graphics; app.status_line = format!("Graphics: {}", if app.enhanced_graphics { "Enhanced" } else { "Normal" }); }

        _ => {}
    }
    true
}

fn handle_insert_mode(app: &mut App, code: KeyCode) -> bool {
    match (app.insert_field, code) {
        // Global controls
        (_, KeyCode::Esc) => {
            app.input_mode = InputMode::Normal;
            app.status_line = "Cancelled".into();
        }
        (_, KeyCode::Enter) => {
            let title = app.draft_title.trim();
            let notes = app.draft_notes.trim();
            let tf = app.draft_timeframe.trim();
            if !title.is_empty() {
                let notes_opt = (!notes.is_empty()).then(|| notes.to_string());
                let tf_opt = (!tf.is_empty()).then(|| tf.to_string());
                app.list.add(title, app.draft_priority, notes_opt); // push to bottom
                if let Some(last) = app.list.items.last_mut() {
                    last.timeframe = tf_opt;
                }
                app.status_line = "Added ✓".to_string();
                app.dirty = true;
                app.input_mode = InputMode::Normal;
            } else {
                app.status_line = "Title cannot be empty".into();
            }
        }

        // Field navigation (forward only)
        (InsertField::Title, KeyCode::Tab) => app.insert_field = InsertField::Notes,
        (InsertField::Notes, KeyCode::Tab) => app.insert_field = InsertField::Time,
        (InsertField::Time, KeyCode::Tab)  => app.insert_field = InsertField::Priority,
        (InsertField::Priority, KeyCode::Tab) => app.insert_field = InsertField::Title,

        // Text editing for Title / Notes / Time
        (InsertField::Title, KeyCode::Backspace) => { app.draft_title.pop(); }
        (InsertField::Notes, KeyCode::Backspace) => { app.draft_notes.pop(); }
        (InsertField::Time,  KeyCode::Backspace) => { app.draft_timeframe.pop(); }

        (InsertField::Title, KeyCode::Char(c)) => { app.draft_title.push(c); }
        (InsertField::Notes, KeyCode::Char(c)) => { app.draft_notes.push(c); }
        (InsertField::Time,  KeyCode::Char(c)) => { app.draft_timeframe.push(c); }

        // Priority editing with arrows (digits are ignored here)
        (InsertField::Priority, KeyCode::Left)  |
        (InsertField::Priority, KeyCode::Up)    => {
            app.draft_priority = (app.draft_priority - 1).clamp(1, 5);
        }
        (InsertField::Priority, KeyCode::Right) |
        (InsertField::Priority, KeyCode::Down)  => {
            app.draft_priority = (app.draft_priority + 1).clamp(1, 5);
        }

        _ => {}
    }
    true
}
