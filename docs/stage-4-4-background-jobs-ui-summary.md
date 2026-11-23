# Stage 4.4: Background Jobs UI and Toolkit Integration

## Summary

Stage 4.4 adds a live Background Jobs screen to the central Toolkit executable. It reuses the existing `JobManager`, typed event architecture, and shared terminal event loop.

No separate worker loop, UI loop, or terminal ownership was introduced.

## Files created

- `apps/toolkit/src/jobs/view.rs`

## Files modified

- `apps/toolkit/src/app.rs`
  - Added `Screen::BackgroundJobs`.
  - Added `previous_screen` for exact return navigation.
  - Added `job_selected` for stable UI selection.
  - Made the Launcher Background Jobs item available.

- `apps/toolkit/src/events.rs`
  - Added `NavigationTarget::BackgroundJobs`.
  - Added open, return, job selection, cancellation, and demonstration-job events.

- `apps/toolkit/src/event_handler.rs`
  - Added global `F3` handling.
  - Added Background Jobs keyboard controls.
  - Added Launcher entry through the Background Jobs menu item.

- `apps/toolkit/src/update.rs`
  - Preserved the screen opened from before entering Background Jobs.
  - Added job selection, cancellation, and demo submission state transitions.
  - Submits a temporary demo worker using `App::submit_job()`.

- `apps/toolkit/src/renderer.rs`
  - Dispatches rendering to the Background Jobs view.
  - Includes Background Jobs footer shortcuts.

- `apps/toolkit/src/jobs/mod.rs`
  - Exposes the read-only jobs view.

- `apps/toolkit/src/jobs/manager.rs`
  - Returns job snapshots in deterministic `JobId` order for stable selection.

## Navigation

`F3` opens or toggles the Background Jobs screen from Launcher, DayPilot, and Settings.

Pressing `F3` while already in Background Jobs does not create a nested navigation entry.

`Q` or `Esc` restores `previous_screen`:

- Launcher -> Jobs -> Launcher
- DayPilot -> Jobs -> DayPilot
- Settings -> Jobs -> Settings

Toolkit navigation remains centralized in `update()`.

## Jobs screen layout

The read-only view displays:

### Header

- Background Jobs title
- Total registered job count

### Summary

Counts for:

- Running
- Queued
- Completed
- Failed
- Cancelled

### Job list

Each row shows:

- Job name
- Job ID
- Status
- Progress when available

### Job details

The selected snapshot shows:

- Name
- Description
- Job ID
- Status
- Progress
- Started timestamp
- Finished timestamp
- Error, if present

### Footer

Shows navigation, cancellation, demo submission, toggle, and return shortcuts.

## Keyboard controls

- `F3`: Open or toggle Background Jobs.
- `Up` / `j`: Select previous job.
- `Down` / `k`: Select next job.
- `C`: Request cancellation for the selected queued or running job.
- `D`: Submit the temporary demonstration job.
- `Q` / `Esc`: Return to the previous screen.

Cancellation does not block the UI. Terminal and cancelled jobs cannot be cancelled.

## Demonstration job

Pressing `D` submits a temporary job named:

```text
Temporary Demo Job
```

It:

- Reports progress from 0 to 100.
- Sleeps 120 ms between updates.
- Completes after approximately 12 seconds.
- Checks cancellation regularly.
- Returns cleanly when cancelled.

The job is submitted through `App::submit_job()` and uses the existing `JobManager` worker implementation.

## Live updates

The central Toolkit loop continues to:

1. Process pending worker messages.
2. Redraw the active screen.
3. Poll Crossterm input non-blockingly.

This allows progress and terminal status updates while the user is not pressing keys.

## Selection stability

Job snapshots are ordered deterministically by `JobId`.

The selected index is clamped to the current snapshot length by the view. `update()` wraps selection only while at least one job exists, so empty or shortened lists do not select an invalid row.

## Validation

- `cargo check --workspace`: passes.
- `cargo test --workspace`: passes.
- All 8 existing JobManager tests pass.
- `cargo run -p toolkit`: successfully launched.
- The launcher shows Background Jobs as available.
- Application exit restored the terminal and ran manager shutdown.

## Remaining warnings

- `App::job()` is not currently used.
- `JobManager::job()` is not currently used.

These individual-job lookup APIs remain available for future UI or testing needs.

## Scope exclusions

This stage did not add WebScout, persistent storage, background-screen-specific event loops, Tokio, or separate terminal ownership.
