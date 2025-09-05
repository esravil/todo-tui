use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, Tabs, Paragraph, Wrap, BarChart, Gauge, LineGauge, Clear, Table, Row, Cell,
        canvas::{Canvas, Map, MapResolution, Line as CanvasLine},
    },
    text::{Span, Line as TextLine},
    symbols,
};
use crate::app::{App, InputMode, InsertField, MapView};
use crate::task::Status;

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.size();

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

    // INSERT OVERLAY: visible while typing on any tab
    if matches!(app.input_mode, InputMode::Insert) {
        draw_insert_overlay(frame, app, outer[1]);
    }
}

// =================== TAB 0: TODOS ==========================================
fn draw_todos(frame: &mut Frame, app: &App, area: Rect) {
    // If expanded, reserve taller footer; else compact status
    let footer_h = if app.expanded { 7 } else { 3 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(footer_h)].as_ref())
        .split(area);

    // Build rows for Table with two columns: [P#] title | timeframe
    let visible = app.visible_indices();
    let mut rows: Vec<Row> = Vec::with_capacity(visible.len());
    for (list_row, &idx) in visible.iter().enumerate() {
        let t = &app.list.items[idx];
        let left = format!("[P{}] {}", t.priority, t.title);
        let right = t.timeframe.as_deref().unwrap_or("—");
        let mut row = Row::new(vec![
            Cell::from(left),
            Cell::from(Span::styled(right.to_string(), Style::default().fg(Color::Gray))),
        ]);

        // highlight selected row in yellow
        if list_row == app.selected {
            row = row.style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        } else if t.status == Status::Done {
            row = row.style(Style::default().fg(Color::DarkGray));
        }
        rows.push(row);
    }

    // ---- Auto-scroll logic ----
    let viewport_rows = chunks[0].height.saturating_sub(2) as usize;
    let (start, end) = if viewport_rows == 0 || rows.is_empty() {
        (0, rows.len())
    } else {
        let max_start = rows.len().saturating_sub(viewport_rows);
        let mut start = if app.selected >= viewport_rows {
            app.selected + 1 - viewport_rows
        } else {
            0
        };
        if start > max_start { start = max_start; }
        let end = (start + viewport_rows).min(rows.len());
        (start, end)
    };

    // Table::new expects (rows, columns) in your ratatui version.
    let table = Table::new(
        rows[start..end].to_vec(),
        [Constraint::Percentage(70), Constraint::Percentage(30)],
    )
    .block(Block::default().borders(Borders::ALL).title("Todos"))
    .column_spacing(2);

    frame.render_widget(table, chunks[0]);

    // Footer: status (and expanded details if toggled)
    if app.expanded {
        draw_expanded_details(frame, app, chunks[1]);
    } else {
        let help = Paragraph::new(vec![
            TextLine::from("q quit | a add | Enter toggle done | d delete | ↑/↓ move | s save"),
            TextLine::from("Space expand/collapse | Tabs: h/l or ←/→ or [Tab] | t toggle chart | g graphics | m map view"),
            TextLine::from(format!("Status: {}", app.status_line)),
        ])
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(help, chunks[1]);
    }
}

// expanded panel under the list
fn draw_expanded_details(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<TextLine> = Vec::new();
    if let Some(idx) = app.visible_indices().get(app.selected).cloned() {
        let t = &app.list.items[idx];
        lines.push(TextLine::from(Span::styled(
            "Details",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(TextLine::from(format!("Title: {}", t.title)));
        lines.push(TextLine::from(format!("Priority: {}", t.priority)));
        lines.push(TextLine::from(format!("Status: {:?}", t.status)));
        lines.push(TextLine::from(format!(
            "Timeframe: {}",
            t.timeframe.as_deref().unwrap_or("<none>")
        )));
        lines.push(TextLine::from("Notes:"));
        lines.push(TextLine::from(
            t.notes.clone().unwrap_or_else(|| "<none>".to_string()),
        ));
    } else {
        lines.push(TextLine::from("No task selected."));
    }
    let p = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Expanded"));
    frame.render_widget(p, area);
}

// =================== INSERT OVERLAY ========================================
fn draw_insert_overlay(frame: &mut Frame, app: &App, content_area: Rect) {
    // 4 content lines (Title, Notes, Timeframe, Priority) => box height 6 incl. borders
    let box_height = 6;
    let box_width = content_area.width.saturating_sub(4);
    let x = content_area.x + 2;
    let y = content_area.y + content_area.height.saturating_sub(box_height + 1);

    let rect = Rect { x, y, width: box_width, height: box_height };

    // Blink caret for text fields
    let caret_visible = app.pulse.sin() > 0.0;
    let caret = if caret_visible { "▏" } else { " " };

    let label_active = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    let label_inactive = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);

    // Title
    let title_label_style = if matches!(app.insert_field, InsertField::Title) { label_active } else { label_inactive };
    let title_span = if app.draft_title.is_empty() {
        Span::styled("<type a title>", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
    } else {
        Span::raw(app.draft_title.clone())
    };
    let title_line = TextLine::from(vec![
        Span::styled("Title: ", title_label_style),
        title_span,
        if matches!(app.insert_field, InsertField::Title) { Span::raw(caret) } else { Span::raw("") },
    ]);

    // Notes
    let notes_label_style = if matches!(app.insert_field, InsertField::Notes) { label_active } else { label_inactive };
    let notes_span = if app.draft_notes.is_empty() {
        Span::styled("<add optional notes>", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
    } else {
        Span::raw(app.draft_notes.clone())
    };
    let notes_line = TextLine::from(vec![
        Span::styled("Notes: ", notes_label_style),
        notes_span,
        if matches!(app.insert_field, InsertField::Notes) { Span::raw(caret) } else { Span::raw("") },
    ]);

    // Timeframe
    let tf_label_style = if matches!(app.insert_field, InsertField::Time) { label_active } else { label_inactive };
    let tf_span = if app.draft_timeframe.is_empty() {
        Span::styled("<e.g. Today 3–5pm | 2025-09-10 09:00>", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
    } else {
        Span::raw(app.draft_timeframe.clone())
    };
    let tf_line = TextLine::from(vec![
        Span::styled("Time:  ", tf_label_style),
        tf_span,
        if matches!(app.insert_field, InsertField::Time) { Span::raw(caret) } else { Span::raw("") },
    ]);

    // Priority (focusable; arrows adjust)
    let prio_label_style = if matches!(app.insert_field, InsertField::Priority) { label_active } else { label_inactive };
    let mut prio_spans: Vec<Span> = vec![Span::styled("Priority: ", prio_label_style)];
    for n in 1..=5 {
        let active_num = n == app.draft_priority;
        prio_spans.push(Span::styled(
            if active_num { format!("[{}]", n) } else { format!(" {} ", n) },
            if active_num {
                let mut s = Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD);
                if matches!(app.insert_field, InsertField::Priority) {
                    s = s.add_modifier(Modifier::REVERSED);
                }
                s
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        if n != 5 { prio_spans.push(Span::raw(" ")); }
    }
    prio_spans.push(Span::raw("   "));
    prio_spans.push(Span::styled("[←/→ adjust] [Tab switch] [Enter add] [Esc cancel]", Style::default().fg(Color::Gray)));
    let prio_line = TextLine::from(prio_spans);

    frame.render_widget(Clear, rect);
    let panel = Paragraph::new(vec![title_line, notes_line, tf_line, prio_line])
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title("Add Task"));
    frame.render_widget(panel, rect);
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
            .constraints([Constraint::Length(7), Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(chunks[0]);

        let pct = app.percent_done();
        let anim = app.progress;
        let label = format!("Done: {:>5.1}%", pct * 100.0);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Completion"))
            .gauge_style(Style::default().fg(Color::Magenta).bg(Color::Black).add_modifier(Modifier::BOLD))
            .label(label)
            .ratio(((pct * 0.85) + (anim * 0.15)).clamp(0.0, 1.0));
        frame.render_widget(gauge, left[0]);

        let lg = LineGauge::default()
            .block(Block::default().borders(Borders::ALL).title("Focus"))
            .filled_style(Style::default().fg(Color::Cyan))
            .line_set(if app.enhanced_graphics { symbols::line::THICK } else { symbols::line::NORMAL })
            .ratio((0.5 + 0.5 * app.pulse.sin()).clamp(0.0, 1.0));
        frame.render_widget(lg, left[1]);

        let sp = ratatui::widgets::Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("Activity"))
            .style(Style::default().fg(Color::Green))
            .data(&app.spark_points)
            .bar_set(if app.enhanced_graphics { symbols::bar::NINE_LEVELS } else { symbols::bar::THREE_LEVELS });
        frame.render_widget(sp, left[2]);
    }

    if app.show_chart {
        let counts = app.counts_by_priority();
        let data: Vec<(&str, u64)> = ["P1","P2","P3","P4","P5"].iter().zip(counts).map(|(l,v)| (*l, v)).collect();

        let bc = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title("By Priority"))
            .data(&data)
            .bar_width(4)
            .bar_gap(1)
            .bar_set(if app.enhanced_graphics { symbols::bar::NINE_LEVELS } else { symbols::bar::THREE_LEVELS })
            .value_style(Style::default().fg(Color::Black).bg(Color::Green))
            .label_style(Style::default().fg(Color::Yellow))
            .bar_style(Style::default().fg(Color::Green));
        frame.render_widget(bc, chunks[1]);
    }
}

// =================== TAB 2: WORLD / NYC ====================================
fn draw_world(frame: &mut Frame, app: &App, area: Rect) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(area);

    // small selector header inside the tab
    let view_titles: Vec<TextLine> = ["World", "NYC"]
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let active = matches!((i, app.map_view), (0, MapView::World) | (1, MapView::NYC));
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

    let selected_idx = match app.map_view { MapView::World => 0, MapView::NYC => 1 };
    let selector = Tabs::new(view_titles)
        .block(Block::default().borders(Borders::ALL).title("Map View — press m"))
        .select(selected_idx)
        .highlight_style(Style::default().fg(Color::Yellow));
    frame.render_widget(selector, sections[0]);

    match app.map_view {
        MapView::World => draw_world_global(frame, app, sections[1]),
        MapView::NYC => draw_world_nyc(frame, app, sections[1]),
    }
}

fn draw_world_global(frame: &mut Frame, app: &App, area: Rect) {
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
        .marker(if app.enhanced_graphics { symbols::Marker::Braille } else { symbols::Marker::Dot })
        .paint(|ctx| {
            ctx.draw(&Map {
                color: Color::White,
                resolution: if app.enhanced_graphics { MapResolution::High } else { MapResolution::Low },
            });

            ctx.draw(&CanvasLine { x1: nyc.1, y1: nyc.0, x2: par.1, y2: par.0, color: Color::Gray });
            ctx.draw(&CanvasLine { x1: nyc.1, y1: nyc.0, x2: sgp.1, y2: sgp.0, color: Color::Gray });
            ctx.print(moving_lon, moving_lat, Span::styled("•", Style::default().fg(Color::White)));
        });

    frame.render_widget(canvas, area);
}

fn draw_world_nyc(frame: &mut Frame, app: &App, area: Rect) {
    // If we loaded real NYC paths (from JSON or GeoJSON), draw them neutrally.
    if let (Some(paths), Some((lon_minmax, lat_minmax))) = (&app.nyc_paths, &app.nyc_bbox) {
        let x_bounds = *lon_minmax;
        let y_bounds = *lat_minmax;

        let canvas = Canvas::default()
            .block(Block::default().title("NYC").borders(Borders::ALL))
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .marker(if app.enhanced_graphics { symbols::Marker::Braille } else { symbols::Marker::Dot })
            .paint(|ctx| {
                for feature in &paths.0 {
                    for ring in feature {
                        // draw ring as series of short white line segments
                        for seg in ring.windows(2) {
                            let a = seg[0]; let b = seg[1];
                            ctx.draw(&CanvasLine { x1: a[0], y1: a[1], x2: b[0], y2: b[1], color: Color::White });
                        }
                        // close the ring
                        if ring.len() > 2 {
                            let a = ring[ring.len()-1]; let b = ring[0];
                            ctx.draw(&CanvasLine { x1: a[0], y1: a[1], x2: b[0], y2: b[1], color: Color::White });
                        }
                    }
                }

                // Neutral labels (adjusted positions)
                ctx.print(-74.01, 40.82, Span::styled("Manhattan",     Style::default().fg(Color::White).add_modifier(Modifier::BOLD))); // down & left
                ctx.print(-73.88, 40.85, Span::styled("Bronx",         Style::default().fg(Color::White)));                                 // to the right
                ctx.print(-73.84, 40.70, Span::styled("Queens",        Style::default().fg(Color::White)));
                ctx.print(-73.97, 40.64, Span::styled("Brooklyn",      Style::default().fg(Color::White)));                                 // more down
                ctx.print(-74.20, 40.60, Span::styled("Staten Island", Style::default().fg(Color::White)));
            });

        frame.render_widget(canvas, area);
        return;
    }

    // Fallback: simple diagrammatic outlines in white (no connector lines, no colors)
    let x_bounds = [-74.30, -73.65];
    let y_bounds = [40.45, 40.95];

    let canvas = Canvas::default()
        .block(Block::default().title("NYC (simple)").borders(Borders::ALL))
        .x_bounds(x_bounds)
        .y_bounds(y_bounds)
        .marker(if app.enhanced_graphics { symbols::Marker::Braille } else { symbols::Marker::Dot })
        .paint(|ctx| {
            let manh = [
                (-74.02, 40.70), (-74.02, 40.88), (-73.93, 40.88), (-73.93, 40.70), (-74.02, 40.70)
            ];
            let brook = [
                (-74.05, 40.57), (-74.05, 40.73), (-73.85, 40.73), (-73.85, 40.57), (-74.05, 40.57)
            ];
            let queens = [
                (-73.96, 40.54), (-73.96, 40.80), (-73.70, 40.80), (-73.70, 40.54), (-73.96, 40.54)
            ];
            let bronx = [
                (-73.93, 40.79), (-73.93, 40.91), (-73.77, 40.91), (-73.77, 40.79), (-73.93, 40.79)
            ];
            let staten = [
                (-74.25, 40.48), (-74.25, 40.65), (-74.05, 40.65), (-74.05, 40.48), (-74.25, 40.48)
            ];

            let outlines = [&manh, &brook, &queens, &bronx, &staten];
            for ring in outlines {
                for seg in ring.windows(2) {
                    let (a, b) = (seg[0], seg[1]);
                    ctx.draw(&CanvasLine { x1: a.0, y1: a.1, x2: b.0, y2: b.1, color: Color::White });
                }
            }

            // Neutral labels (adjusted positions)
            ctx.print(-74.01, 40.82, Span::styled("Manhattan",     Style::default().fg(Color::White).add_modifier(Modifier::BOLD))); // down & left
            ctx.print(-73.90, 40.88, Span::styled("Bronx",         Style::default().fg(Color::White)));                                 // to the right
            ctx.print(-73.93, 40.70, Span::styled("Queens",        Style::default().fg(Color::White)));
            ctx.print(-74.03, 40.64, Span::styled("Brooklyn",      Style::default().fg(Color::White)));                                 // more down
            ctx.print(-74.20, 40.60, Span::styled("Staten Island", Style::default().fg(Color::White)));
        });

    frame.render_widget(canvas, area);
}
