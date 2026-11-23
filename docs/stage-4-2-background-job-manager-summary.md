# Stage 4.2: Background Job Manager

## Summary

Stage 4.2 adds a standard-library Background Job Manager for independently executing jobs, reporting progress, tracking lifecycle state, supporting cooperative cancellation, and joining worker threads.

It remains independent of `App`, Toolkit navigation, event handling, and UI rendering.

## Files changed

### Created

- `apps/toolkit/src/jobs/manager.rs`

### Modified

- `apps/toolkit/src/jobs/mod.rs`
  - Declares the manager module.
  - Re-exports `JobManager` and `JobContext`.

The existing `model.rs` types were left unchanged.

## Public API

### `JobManager`

- `new()` creates an empty manager.
- `submit(name, description, work)` starts a job and returns a unique `JobId`.
- `request_cancellation(id)` requests cooperative cancellation.
- `process_pending_messages()` processes all currently available worker messages without blocking.
- `jobs()` returns snapshots of every registered job.
- `job(id)` returns a snapshot of one job.
- `shutdown()` requests cancellation and joins worker threads.

### `JobContext`

Passed to every submitted closure:

- `report_progress(progress)` reports a percentage from `0` through `100`.
- `is_cancelled()` checks whether cancellation was requested.

## Worker communication

Workers communicate with the manager through `std::sync::mpsc` messages:

- `Started`
- `Progress`
- `Completed`
- `Failed`
- `Cancelled`

Workers never mutate the manager registry directly. The manager receives messages and is the only component that updates `Job` records.

## Cancellation

Each job receives its own `Arc<AtomicBool>` cancellation signal.

A worker cooperatively checks `JobContext::is_cancelled()` and exits cleanly when requested. The cancellation request is not a forced termination.

If cancellation or completion wins the race safely:

- Cancellation is represented as `Cancelled` only for queued or running jobs.
- Terminal states are not overwritten by late progress or completion messages.

## Thread ownership

The manager owns a `JoinHandle<()>` for every worker thread. It does not detach threads.

`shutdown()`:

1. Requests cancellation for queued and running jobs.
2. Processes currently available messages.
3. Joins each owned thread handle.
4. Processes messages after each join.
5. Marks any remaining queued or running jobs cancelled.

Normal TUI event loops are not blocked by workers themselves. `shutdown()` may block while joining threads.

Cooperative cancellation means a worker that never checks `is_cancelled()` can delay shutdown.

## Lifecycle protection

Only the expected transitions are applied:

- `Queued -> Running`
- `Running -> Completed`
- `Running -> Failed`
- `Running -> Cancelled`

Terminal states prevent:

- Late progress updates
- Late failures
- Late completions
- Duplicate completion messages

## Tests

The following eight unit tests were added:

1. Unique IDs for submitted jobs
2. Successful job completion
3. Failed job execution
4. Progress reporting and clamping
5. Cooperative cancellation
6. Job retrieval by ID
7. Multiple concurrent jobs
8. Terminal-state protection

Tests use channels, barriers, bounded polling, and short polling intervals instead of fixed sleep assumptions.

## Validation

- `cargo check --workspace`: passes
- `cargo test --workspace`: passes
- 8 JobManager tests passed
- DayPilot and Toolkit tests also passed or reported zero tests

## Known limitations

- The manager is not yet stored in `App`.
- No Toolkit event or UI integration exists yet.
- `process_pending_messages()` must be called by future application integration.
- Worker work may panic; it is converted to a failed job when the panic is catchable by the worker wrapper.
- A worker that never checks cancellation can delay `shutdown()`.

No application navigation, UI, DayPilot, WebScout, or future-stage behavior was implemented.
