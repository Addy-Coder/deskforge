# Stage 4.1: Background Job Data Model

## Summary

Stage 4.1 introduces only the foundational data model for the future Background Job Manager.

No worker, channel, execution, cancellation, manager, UI, or application-state behavior was implemented.

## New files

```text
apps/toolkit/src/jobs/
├── mod.rs
└── model.rs
```

### `jobs/mod.rs`

Declares the private model module and re-exports the public job types:

- `Job`
- `JobId`
- `JobStatus`

### `jobs/model.rs`

Defines the job data structures.

#### `JobId`

A lightweight type-safe wrapper around `u64`:

```rust
pub struct JobId(pub u64);
```

Derived traits:

- `Clone`
- `Copy`
- `Debug`
- `PartialEq`
- `Eq`
- `Hash`

#### `JobStatus`

A standalone status enum:

- `Queued`
- `Running`
- `Completed`
- `Failed`
- `Cancelled`

Derived traits:

- `Clone`
- `Copy`
- `Debug`
- `PartialEq`
- `Eq`

#### `Job`

Contains:

- `id: JobId`
- `name: String`
- `description: String`
- `status: JobStatus`
- `progress: Option<u8>`
- `started_at: Option<DateTime<Local>>`
- `finished_at: Option<DateTime<Local>>`
- `error: Option<String>`

Derived traits:

- `Clone`
- `Debug`
- `PartialEq`

## Modified files

### `apps/toolkit/src/main.rs`

Declared:

```rust
mod jobs;
```

The module is compiled but is not currently integrated into `App`, events, updates, or rendering.

### `apps/toolkit/Cargo.toml`

Added:

```toml
chrono = "0.4"
```

## Design decisions

- `JobId` is a tuple wrapper around `u64`, keeping it lightweight and preventing accidental mixing with unrelated integers.
- `progress` remains optional because some jobs may not expose percentage information.
- `started_at` and `finished_at` are optional to represent queued and unfinished jobs.
- `error` is optional and is meaningful only for failed jobs.
- `JobStatus` is intentionally independent rather than storing state-specific nested data.

## Verification

`cargo check --workspace` passes.

## Warnings

The compiler reports dead-code and unused re-export warnings because this stage creates the model without constructing jobs or integrating the module into application state or UI.

These warnings are expected and will be resolved when the Job Manager and UI are implemented.

## Excluded from this stage

- Worker threads
- Message-passing channels
- Job execution
- Cancellation
- Job Manager state
- Background job UI
- Keyboard shortcuts
- App event variants
- Changes to DayPilot
