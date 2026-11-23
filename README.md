# DeskForge

**Your terminal. Your tools. Your command center.**

DeskForge is a terminal-based personal command center built with Rust. It combines task management, application settings, and observable background jobs in a single Ratatui interface.

## Current Features

- Central launcher with in-memory application navigation
- DayPilot task management:
  - View and select tasks
  - Add and edit tasks
  - Delete tasks with confirmation
  - Change completion status
  - Set priority and deadline fields
- Application theme settings:
  - Burnt Ember
  - High Contrast
- Background Jobs monitor:
  - Job status and progress snapshots
  - Job details and errors
  - Cooperative cancellation
  - Temporary demonstration job submission
- Previous-screen restoration when leaving Background Jobs
- Shared terminal initialization, event loop, and shutdown lifecycle

## Planned Features

The launcher currently identifies WebScout and SysPeek as planned modules. They are not implemented yet.

## Technology Stack

- Rust 2024 edition
- Ratatui 0.29
- Crossterm 0.28
- Chrono 0.4
- anyhow
- Standard-library threads and `mpsc` channels

## Architecture

DeskForge is a modular monolith: one Toolkit process owns the terminal, central application state, and main event loop. Modules communicate through typed application events and centralized update logic.

The Workspace contains:

- `toolkit`: the main executable and application orchestrator
- `daypilot`: the reusable task-management library and standalone executable

Key Toolkit layers include application state, event definitions, keyboard event handling, updates, rendering, and the background job module.

## Repository Structure

```text
personal-toolkit/
├── Cargo.toml
├── Cargo.lock
├── apps/
│   ├── toolkit/
│   │   └── src/
│   │       ├── app.rs
│   │       ├── event_handler.rs
│   │       ├── events.rs
│   │       ├── renderer.rs
│   │       ├── update.rs
│   │       └── jobs/
│   └── daypilot/
│       └── src/
│           ├── lib.rs
│           ├── model.rs
│           ├── events.rs
│           ├── event_handler.rs
│           ├── update.rs
│           ├── view.rs
│           └── main.rs
└── docs/
```

## Prerequisites

- Rust toolchain with Cargo
- A terminal capable of alternate-screen and raw-mode TUI applications

## Build and Run

Build the workspace:

```powershell
cargo check --workspace
```

Run the main DeskForge Toolkit:

```powershell
cargo run -p toolkit
```

Run the standalone DayPilot executable:

```powershell
cargo run -p daypilot
```

## Keyboard Shortcuts

### Global

- `Ctrl+Q`: Exit DeskForge
- `F3`: Open or toggle Background Jobs

### Launcher

- `Up` / `j`: Previous item
- `Down` / `k`: Next item
- `Enter`: Open the selected application
- `Q`: Exit

### DayPilot

- `Up` / `j`: Previous task
- `Down` / `k`: Next task
- `A`: Add task
- `E`: Edit selected task
- `D`: Delete selected task
- `Space`: Toggle task completion
- `Q` / `Esc`: Return to the previous Toolkit screen
- `Tab` / `Shift+Tab`: Navigate form fields
- `Left` / `Right`: Change priority in the form
- `Enter`: Save form
- `Esc`: Cancel form
- `Y`: Confirm deletion
- `N` / `Esc`: Cancel deletion

### Settings

- `Left` / `Right` / `Enter`: Change theme
- `Q` / `Esc`: Return to the launcher

### Background Jobs

- `Up` / `j`: Previous job
- `Down` / `k`: Next job
- `C`: Request cancellation for the selected queued or running job
- `D`: Submit a temporary demonstration job
- `F3`: Toggle this screen
- `Q` / `Esc`: Return to the previous screen

## Testing

Run all workspace tests:

```powershell
cargo test --workspace
```

Format source files:

```powershell
cargo fmt --all
```

Check formatting:

```powershell
cargo fmt --all -- --check
```

## Project Status

DeskForge is in active development. DayPilot, Settings, theme switching, navigation, and the background job manager/UI are implemented. WebScout and SysPeek remain planned.
