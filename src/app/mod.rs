pub mod ui;
pub mod input;

use crate::task::Status;
use crate::todolist::TodoList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Active,
    Done,
}

/// Central application state for the TUI.
/// Rendering reads from this; input mutates this.
pub struct App {
    pub list: TodoList,
    pub selected: usize,
    pub input_mode: InputMode,
    pub filter: Filter,

    // Draft fields for Insert mode
    pub draft_title: String,
    pub draft_priority: i8,
    pub draft_notes: String,

    pub status_line: String,
    pub dirty: bool, // when true, main loop will persist to disk
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
        }
    }

    /// Visible indices after applying the current filter.
    pub fn visible_indices(&self) -> Vec<usize> {
        self.list
            .items
            .iter()
            .enumerate()
            .filter(|(_, t)| match self.filter {
                Filter::All => true,
                Filter::Active => t.status == Status::Pending,
                Filter::Done => t.status == Status::Done,
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn select_next(&mut self) {
        let len = self.visible_indices().len();
        if len > 0 && self.selected + 1 < len {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Keep the selected row within bounds after deletions/filter changes.
    pub fn clamp_selection(&mut self) {
        let len = self.visible_indices().len();
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }

    /// Cycle filter: All → Active → Done → All
    pub fn cycle_filter(&mut self) {
        self.filter = match self.filter {
            Filter::All => Filter::Active,
            Filter::Active => Filter::Done,
            Filter::Done => Filter::All,
        };
        self.status_line = format!("Filter: {:?}", self.filter);
        self.clamp_selection();
    }
}
