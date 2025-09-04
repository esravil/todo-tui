pub mod ui;
pub mod input;

use crate::task::Status;
use crate::todolist::TodoList;

// Tabs
#[derive(Debug)]
pub struct Tabs {
    pub titles: Vec<&'static str>,
    pub index: usize,
}
impl Tabs {
    pub fn new(titles: Vec<&'static str>) -> Self {
        Self { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }
    pub fn prev(&mut self) {
        self.index = if self.index == 0 { self.titles.len() - 1 } else { self.index - 1 };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode { Normal, Insert }

// Focusable fields in Insert mode (Tab cycles through these)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertField { Title, Notes, Time, Priority }

/// Central TUI state
pub struct App {
    pub list: TodoList,
    pub selected: usize,
    pub input_mode: InputMode,

    // insert-mode drafts
    pub insert_field: InsertField,
    pub draft_title: String,
    pub draft_priority: i8,
    pub draft_notes: String,
    pub draft_timeframe: String,

    pub status_line: String,
    pub dirty: bool,

    // visuals/animation
    pub tabs: Tabs,
    pub show_chart: bool,
    pub enhanced_graphics: bool,
    pub progress: f64, // 0..1 wave
    pub pulse: f64,    // 0..tau loop
    pub spark_points: Vec<u64>,

    // inline expansion in Todos tab
    pub expanded: bool,
}

impl App {
    pub fn new(list: TodoList) -> Self {
        Self {
            list,
            selected: 0,
            input_mode: InputMode::Normal,
            insert_field: InsertField::Title,
            draft_title: String::new(),
            draft_priority: 1,
            draft_notes: String::new(),
            draft_timeframe: String::new(),
            status_line: String::new(),
            dirty: false,

            // Details tab removed; inline expansion instead
            tabs: Tabs::new(vec!["Todos", "Dash", "World"]),
            show_chart: true,
            enhanced_graphics: true,

            progress: 0.0,
            pulse: 0.0,
            spark_points: vec![0; 60],

            expanded: false,
        }
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        (0..self.list.items.len()).collect()
    }
    pub fn select_next(&mut self) {
        let len = self.visible_indices().len();
        if len > 0 && self.selected + 1 < len { self.selected += 1; }
    }
    pub fn select_prev(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }
    pub fn clamp_selection(&mut self) {
        let len = self.visible_indices().len();
        if len == 0 { self.selected = 0; }
        else if self.selected >= len { self.selected = len - 1; }
    }

    // metrics
    pub fn percent_done(&self) -> f64 {
        let total = self.list.items.len() as f64;
        if total == 0.0 { 0.0 } else {
            let done = self.list.items.iter().filter(|t| t.status == Status::Done).count() as f64;
            done / total
        }
    }
    pub fn counts_by_priority(&self) -> [u64; 5] {
        let mut c = [0u64; 5];
        for t in &self.list.items {
            let p = t.priority.clamp(1, 5) as usize;
            c[p - 1] += 1;
        }
        c
    }

    // animation tick
    pub fn on_tick(&mut self) {
        self.progress += 0.01;
        if self.progress > 1.0 { self.progress = 0.0; }

        let base = (self.percent_done() * 100.0) as u64;
        let wobble = ((self.pulse.sin() * 20.0) + 20.0) as u64;
        self.spark_points.remove(0);
        self.spark_points.push(base + wobble);

        const TAU: f64 = std::f64::consts::PI * 2.0;
        self.pulse += 0.07;
        if self.pulse > TAU { self.pulse -= TAU; }
    }
}
