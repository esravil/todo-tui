use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, Tabs, Paragraph, Wrap, List, ListItem, BarChart, Gauge, LineGauge, Clear,
        canvas::{Canvas, Map, MapResolution, Line as CanvasLine},
    },
    text::{Span, Line as TextLine},
    symbols,
};
use super::{App, InputMode};
use crate::task::Status;

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(size);

    // Tabs header
    let titles: Vec<TextLine> = app
        .tabs
        .titles
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let active = i == app.tabs.index;
            TextLine::from(Span::styled(
                *t,
                if active {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                },
            ))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("todo-tui"))
        .select(app.tabs.index)
        .highlight_style(Style::default().fg(Color::Yellow));
    frame.render_widget(tabs, outer[0]);

    // Main content by tab
    match app.tabs.index {
        0 => draw_todos(frame, app, outer[1]),
        1 => draw_dash(frame, app, outer[1]),
        2 => draw_world(frame, app, outer[1]),
        _ => {}
    }

    // INSERT OVERLAY: always visible while typing, regardless of tab
    if matches!(app.input_mode, InputMode::Insert) {
        draw_insert_overlay(frame, app, outer[1]);
    }
}

// =================== TAB 0: TODOS ==========================================
fn draw_todos(frame: &mut Frame, app: &App, area: Rect) {
    // list + footer (footer only in Normal mode; Insert uses overlay)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(area);

    // Main list
    let visible = app.visible_indices();
    let items: Vec<ListItem> = visible
        .iter()
        .map(|&idx| {
            let t = &app.list.items[idx];
            let p = Span::styled(
                format!("[P{}] ", t.priority),
                Style::default().add_modifier(Modifier::BOLD),
            );
            let style = if t.status == Status::Done {
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default()
            };
            let title = Span::styled(t.title.clone(), style);
            ListItem::new(TextLine::from(vec![p, title]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Todos"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");

    let mut state = ratatui::widgets::ListState::default();
    if !visible.is_empty() {
        state.select(Some(app.selected));
    }
    frame.render_stateful_widget(list, chunks[0], &mut state);

    // Footer help only in Normal mode (Insert uses overlay)
    if matches!(app.input_mode, InputMode::Normal) {
        let help = Paragraph::new(vec![
            TextLine::from("q quit | a add | space toggle | d delete | up/down move | s save"),
            TextLine::from("Tabs: h/l or ←/→ or [Tab] | t toggle chart | g graphics"),
            TextLine::from(format!("Status: {}", app.status_line)),
        ])
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(help, chunks[1]);
    }
}

// =================== INSERT OVERLAY (shown on all tabs) ====================
fn draw_insert_overlay(frame: &mut Frame, app: &App, content_area: Rect) {
    // Two content lines (Title + Priority) => 2 + 2 borders = 4 rows
    let box_height = 4;
    let box_width = content_area.width.saturating_sub(4);
    let x = content_area.x + 2;
    let y = content_area
        .y
        + content_area
            .height
            .saturating_sub(box_height + 1);

    let rect = Rect {
        x,
        y,
        width: box_width,
        height: box_height,
    };

    // Blink caret using the app's pulse
    let caret_visible = app.pulse.sin() > 0.0;
    let caret = if caret_visible { "▏" } else { " " };

    // Line 1: Title
    let title_span = if app.draft_title.is_empty() {
        Span::styled(
            "<type a title>",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )
    } else {
        Span::raw(app.draft_title.clone())
    };
    let line_title = TextLine::from(vec![
        Span::styled(
            "Title: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        title_span,
        Span::raw(caret),
    ]);

    // Line 2: Priority picker — shows [1] 2 3 4 5 with current highlighted
    let mut prio_spans: Vec<Span> = vec![Span::styled(
        "Priority: ",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )];
    for n in 1..=5 {
        let active = n == app.draft_priority;
        prio_spans.push(Span::styled(
            if active { format!("[{}]", n) } else { format!(" {} ", n) },
            if active {
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        if n != 5 {
            prio_spans.push(Span::raw(" "));
        }
    }
    prio_spans.push(Span::raw("   "));
    prio_spans.push(Span::styled(
        "[1–5 to set | Enter=add | Esc=cancel]",
        Style::default().fg(Color::Gray),
    ));
    let line_priority = TextLine::from(prio_spans);

    // Draw panel
    frame.render_widget(Clear, rect);
    let input_panel = Paragraph::new(vec![line_title, line_priority])
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title("Add Task"));
    frame.render_widget(input_panel, rect);
}

// =================== TAB 1: DASH ===========================================
fn draw_dash(frame: &mut Frame, app: &App, area: Rect) {
    let cols = if app.show_chart {
        vec![Constraint::Percentage(45), Constraint::Percentage(55)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(cols)
        .split(area);

    {
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [Constraint::Length(7), Constraint::Length(3), Constraint::Min(1)].as_ref(),
            )
            .split(chunks[0]);

        let pct = app.percent_done();
        let anim = app.progress;
        let label = format!("Done: {:>5.1}%", pct * 100.0);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Completion"))
            .gauge_style(
                Style::default()
                    .fg(Color::Magenta)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .label(label)
            .ratio(((pct * 0.85) + (anim * 0.15)).clamp(0.0, 1.0));
        frame.render_widget(gauge, left[0]);

        let lg = LineGauge::default()
            .block(Block::default().borders(Borders::ALL).title("Focus"))
            .filled_style(Style::default().fg(Color::Cyan))
            .line_set(if app.enhanced_graphics {
                symbols::line::THICK
            } else {
                symbols::line::NORMAL
            })
            .ratio((0.5 + 0.5 * app.pulse.sin()).clamp(0.0, 1.0));
        frame.render_widget(lg, left[1]);

        let sp = ratatui::widgets::Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("Activity"))
            .style(Style::default().fg(Color::Green))
            .data(&app.spark_points)
            .bar_set(if app.enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            });
        frame.render_widget(sp, left[2]);
    }

    if app.show_chart {
        let counts = app.counts_by_priority();
        let data: Vec<(&str, u64)> = ["P1", "P2", "P3", "P4", "P5"]
            .iter()
            .zip(counts)
            .map(|(l, v)| (*l, v))
            .collect();

        let bc = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title("By Priority"))
            .data(&data)
            .bar_width(4)
            .bar_gap(1)
            .bar_set(if app.enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            })
            .value_style(Style::default().fg(Color::Black).bg(Color::Green))
            .label_style(Style::default().fg(Color::Yellow))
            .bar_style(Style::default().fg(Color::Green));
        frame.render_widget(bc, chunks[1]);
    }
}

// =================== TAB 2: WORLD ==========================================
fn draw_world(frame: &mut Frame, app: &App, area: Rect) {
    let nyc = (40.71_f64, -74.00_f64);
    let sgp = (1.35_f64, 103.86_f64);
    let par = (48.85_f64, 2.35_f64);

    let t = app.pulse.sin() * 0.5 + 0.5;
    let lerp = |a: f64, b: f64| a + (b - a) * t;
    let moving_lat = lerp(nyc.0, sgp.0);
    let moving_lon = lerp(nyc.1, sgp.1);

    let canvas = Canvas::default()
        .block(Block::default().title("World").borders(Borders::ALL))
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
        .marker(if app.enhanced_graphics {
            symbols::Marker::Braille
        } else {
            symbols::Marker::Dot
        })
        .paint(|ctx| {
            ctx.draw(&Map {
                color: Color::White,
                resolution: if app.enhanced_graphics {
                    MapResolution::High
                } else {
                    MapResolution::Low
                },
            });

            ctx.draw(&CanvasLine {
                x1: nyc.1,
                y1: nyc.0,
                x2: par.1,
                y2: par.0,
                color: Color::Yellow,
            });
            ctx.draw(&CanvasLine {
                x1: nyc.1,
                y1: nyc.0,
                x2: sgp.1,
                y2: sgp.0,
                color: Color::Gray,
            });
            ctx.print(
                moving_lon,
                moving_lat,
                Span::styled("•", Style::default().fg(Color::Magenta)),
            );
        });

    frame.render_widget(canvas, area);
}
