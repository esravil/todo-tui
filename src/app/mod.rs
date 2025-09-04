pub mod ui;
pub mod input;

use crate::task::Status;
use crate::todolist::TodoList;

// === NEW: tabs ==============================================================
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter { All, Active, Done }

/// Central TUI state (UI reads; input mutates)
pub struct App {
    pub list: TodoList,
    pub selected: usize,
    pub input_mode: InputMode,
    pub filter: Filter,

    // insert-mode drafts
    pub draft_title: String,
    pub draft_priority: i8,
    pub draft_notes: String,

    pub status_line: String,
    pub dirty: bool,

    // === NEW: visual/animation state =======================================
    pub tabs: Tabs,
    pub show_chart: bool,
    pub enhanced_graphics: bool,

    // gauge anim (progress waves 0..1); also used as a phase for the map
    pub progress: f64,
    pub pulse: f64,         // 0..tau looping

    // sparkline points (length ~60)
    pub spark_points: Vec<u64>,
}

impl App {
    pub fn new(list: TodoList) -> Self {
        Self {
            list,
            selected: 0,
            input_mode: InputMode::Normal,
            filter: Filter::All,
            draft_title: String::new(),
            draft_priority: 1,
            draft_notes: String::new(),
            status_line: String::new(),
            dirty: false,

            // visuals
            tabs: Tabs::new(vec!["Todos", "Dash", "World"]),
            show_chart: true,
            enhanced_graphics: true,

            progress: 0.0,
            pulse: 0.0,
            spark_points: vec![0; 60],
        }
    }

    // ----- existing helpers -----
    pub fn visible_indices(&self) -> Vec<usize> {
        self.list.items.iter().enumerate().filter(|(_, t)| match self.filter {
            Filter::All => true,
            Filter::Active => t.status == Status::Pending,
            Filter::Done => t.status == Status::Done,
        }).map(|(i, _)| i).collect()
    }
    pub fn select_next(&mut self) { let len = self.visible_indices().len(); if len > 0 && self.selected + 1 < len { self.selected += 1; } }
    pub fn select_prev(&mut self) { if self.selected > 0 { self.selected -= 1; } }
    pub fn clamp_selection(&mut self) { let len = self.visible_indices().len(); if len == 0 { self.selected = 0; } else if self.selected >= len { self.selected = len - 1; } }
    pub fn cycle_filter(&mut self) {
        self.filter = match self.filter { Filter::All => Filter::Active, Filter::Active => Filter::Done, Filter::Done => Filter::All };
        self.status_line = format!("Filter: {:?}", self.filter);
        self.clamp_selection();
    }

    // ----- NEW: metrics derived from tasks ---------------------------------
    pub fn percent_done(&self) -> f64 {
        let total = self.list.items.len() as f64;
        if total == 0.0 { 0.0 } else {
            let done = self.list.items.iter().filter(|t| t.status == Status::Done).count() as f64;
            done / total
        }
    }
    /// Count tasks by priority 1..=5, clamped into [1,5]
    pub fn counts_by_priority(&self) -> [u64; 5] {
        let mut c = [0u64; 5];
        for t in &self.list.items {
            let p = t.priority.clamp(1, 5) as usize;
            c[p - 1] += 1;
        }
        c
    }

    // ----- NEW: animation tick (call ~10–20 times/sec) ---------------------
    pub fn on_tick(&mut self) {
        // progress wave
        self.progress += 0.01;
        if self.progress > 1.0 { self.progress = 0.0; }

        // sparkline: shift + push a deterministic “wavy” load reading
        let base = (self.percent_done() * 100.0) as u64;
        let wobble = ((self.pulse.sin() * 20.0) + 20.0) as u64;
        let val = base + wobble;
        self.spark_points.remove(0);
        self.spark_points.push(val);

        // world map animation phase
        const TAU: f64 = std::f64::consts::PI * 2.0;
        self.pulse += 0.07;
        if self.pulse > TAU { self.pulse -= TAU; }
    }
}
