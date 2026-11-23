# Stage 4.3: Integrate JobManager into Toolkit

## Summary

Stage 4.3 connects the standalone `JobManager` to the central Toolkit application while preserving navigation, themes, DayPilot, and terminal behavior.

No background-job UI or keyboard shortcuts were added.

## Files modified

### `apps/toolkit/src/app.rs`

Added persistent ownership:

```rust
jobs: JobManager
```

`App::new()` creates one manager alongside the persistent `DayPilotApp` state.

Added App-level job coordination methods:

- `submit_job(...)`
- `request_job_cancellation(...)`
- `process_job_messages()`
- `jobs()`
- `job(...)`
- `shutdown_jobs()`

The `JobManager` remains private to `App`; unrelated modules receive snapshots or use the App wrappers.

### `apps/toolkit/src/main.rs`

Added message processing to every Toolkit event-loop iteration:

```rust
app.process_job_messages();
```

This call occurs before drawing and is independent of Crossterm key polling.

Added shutdown after the application loop exits:

```rust
app.shutdown_jobs();
```

This occurs before `main()` restores the terminal.

### `apps/toolkit/src/jobs/manager.rs`

Shutdown now has an internal guard:

```rust
shutdown_started: bool
```

Repeated calls to `shutdown()` return immediately after the first call.

## Pending message processing

`run_app()` processes job messages every loop iteration:

- It does not wait for a worker message.
- It uses `try_recv()` inside `JobManager::process_pending_messages()`.
- Crossterm polling remains responsive.
- Messages continue to be processed even if no key is pressed.

## Lifecycle integration

### Navigation

Navigating between launcher, Settings, and DayPilot does not recreate or shut down the JobManager.

The manager remains alive for the lifetime of `App`.

### Application exit

When `app.running` becomes false:

1. The main Toolkit loop exits.
2. `app.shutdown_jobs()` is called.
3. Active jobs receive cooperative cancellation requests.
4. Worker thread handles are joined.
5. Control returns to `main()`.
6. The terminal restores raw-mode and alternate-screen state.

Ordinary screen transitions do not invoke shutdown.

## Shutdown behavior

`JobManager::shutdown()`:

- Requests cancellation for every registered queued or running job.
- Processes pending messages.
- Drains owned worker `JoinHandle`s.
- Joins each worker thread explicitly.
- Processes worker messages after joining.
- Marks unfinished jobs cancelled if necessary.

Shutdown is cooperative and idempotent. It may block while joining workers, particularly if a worker ignores cancellation checks.

## Test and validation results

- `cargo check --workspace`: passes.
- `cargo test --workspace`: passes.
- All 8 JobManager tests pass.
- DayPilot, standalone executables, and documentation tests pass or contain zero tests.

## Warnings

Warnings indicate that submission and snapshot App methods are not yet called because this stage adds no UI or worker-producing module.

Other warnings reflect unused public job types and context methods until future Toolkit integration.

These warnings do not indicate compilation failures.

## Scope exclusions

This stage does not add:

- A Background Jobs UI
- F3 handling
- Toolkit event variants for jobs
- Job submission from existing modules
- WebScout
- DayPilot changes
- Persistent storage
- Tokio or another concurrency framework
