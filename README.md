# todo-tui

A simple **terminal user interface (TUI) todo list manager** written in Rust.  
It combines [`clap`](https://crates.io/crates/clap) for CLI parsing and [`ratatui`](https://crates.io/crates/ratatui) for a styled terminal UI.

---

## Features
- Add, delete, and mark tasks as done
- Priorities from 1–5
- Animated gauges, sparklines, and a world map demo
- Tabbed interface:
  - **Todos** – main list
  - **Dashboard** – progress gauges, bar chart, sparkline
  - **World** – animated map view
- Tasks are persisted in a file for simplicity

---

## Installation & Running
```bash
# clone repo

# run with cargo
cargo run
