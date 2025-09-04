use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::{App, Filter, InputMode};
use crate::task::Status;

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // header
                Constraint::Min(1),    // list
                Constraint::Length(3), // footer
            ]
            .as_ref(),
        )
        .split(size);

    // Header (help + filter)
    let help = Line::from("q quit  a add  d delete  <space> toggle  j/k move  / filter  s save");
    let filt = Line::from(match app.filter {
        Filter::All => "Filter: All",
        Filter::Active => "Filter: Active",
        Filter::Done => "Filter: Done",
    });
    let header = Paragraph::new(vec![help, filt])
        .block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(header, chunks[0]);

    // Main list
    let visible = app.visible_indices();
    let items: Vec<ListItem> = visible
        .iter()
        .map(|&idx| {
            let t = &app.list.items[idx];
            let priority = Span::styled(
                format!("[P{}] ", t.priority),
                Style::default().add_modifier(Modifier::BOLD),
            );
            let title_style = if t.status == Status::Done {
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default()
            };
            let title = Span::styled(t.title.clone(), title_style);

            let mut line = vec![priority, title];
            if let Some(notes) = &t.notes {
                if !notes.is_empty() {
                    line.push(Span::raw("  — "));
                    line.push(Span::styled(
                        notes.clone(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }
            ListItem::new(Line::from(line))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Todos"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !visible.is_empty() {
        state.select(Some(app.selected));
    }
    frame.render_stateful_widget(list, chunks[1], &mut state);

    // Footer (status or input)
    let (title, content) = match app.input_mode {
        InputMode::Insert => (
            "Add Task",
            format!(
                "Title: {}   [Enter=add | Esc=cancel]   (priority {} via 1 - 5)",
                app.draft_title, app.draft_priority
            ),
        ),
        InputMode::Normal => ("Status", app.status_line.clone()),
    };

    let footer = Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(footer, chunks[2]);
}
