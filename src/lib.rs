pub mod task;
pub mod todolist;
pub mod persistence;
pub mod app;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// CLI shared between main and tests
#[derive(Parser, Debug)]
#[command(name = "todo", version, about = "A small, styled Ratatui todo list")]
pub struct Cli {
    /// Optional override for the data file
    #[arg(long)]
    pub data_file: Option<PathBuf>,

    #[command(subcommand)]
    pub cmd: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Add a task quickly from the CLI
    Add {
        title: String,
        /// Priority (1 highest → larger = lower priority)
        #[arg(short, long, default_value_t = 1)]
        priority: i8,
        /// Optional notes
        #[arg(short, long)]
        notes: Option<String>,
    },
    /// Print all tasks to stdout
    List,
    /// Toggle the 'done' status of a task by its visible index
    Done { index: usize },
    /// Delete a task by its visible index
    Delete { index: usize },
    /// Launch the full-screen TUI
    Tui,
}
