# Stage 3: DayPilot Event-Driven Refactor

## Summary

Stage 3 splits DayPilot into model, event, keyboard conversion, update, and view modules. Toolkit now communicates with DayPilot through typed events instead of the previous Boolean `handle_key()` interface.

## Final DayPilot structure

```text
apps/daypilot/src/
├── lib.rs
├── model.rs
├── events.rs
├── event_handler.rs
├── update.rs
├── view.rs
└── main.rs
```

## File responsibilities

### `model.rs`

Contains DayPilot state and data structures:

- `Task`
- `TaskStatus`
- `DayPilotApp`
- `DayPilotColors`
- Task selection
- Add/edit form state
- Delete confirmation state

Internal fields remain private to the library modules through `pub(crate)` visibility.

### `events.rs`

Defines the strongly typed `DayPilotEvent` enum.

It represents:

- Selection movement
- Task completion
- Starting add, edit, and delete operations
- Delete confirmation and cancellation
- Saving and cancelling forms
- Form field navigation
- Priority changes
- Text entry and deletion
- Returning to the Toolkit launcher

### `event_handler.rs`

Defines:

```rust
pub fn handle_key_event(
    app: &DayPilotApp,
    key: KeyCode,
) -> Option<DayPilotEvent>
```

It interprets keys differently according to:

- Normal task-list mode
- Add/edit form mode
- Delete confirmation mode

It does not mutate DayPilot state.

### `update.rs`

Defines:

```rust
pub fn update(
    app: &mut DayPilotApp,
    event: DayPilotEvent,
)
```

It is the centralized location for DayPilot state changes:

- Selection
- Completion
- Adding and editing
- Saving and cancelling forms
- Deleting and confirmations
- Form fields
- Priority changes
- Text entry

It does not read keyboard input or render UI.

### `view.rs`

Contains the existing DayPilot rendering:

- Header
- Task list
- Task details
- Add/edit form
- Delete confirmation
- Footer

Rendering only reads state and does not modify it.

### `lib.rs`

Exposes only the public library interface:

- `DayPilotApp`
- `DayPilotColors`
- `DayPilotEvent`
- `handle_key_event()`
- `update()`
- `render()`

### `main.rs`

The standalone executable owns its terminal and uses the shared library implementation for event conversion, updates, and rendering.

## Toolkit integration

Toolkit’s `AppEvent` now includes:

```rust
DayPilot(DayPilotEvent)
```

The Toolkit update layer handles this event by:

- Translating `DayPilotEvent::GoBack` into `app.screen = Screen::Launcher`.
- Delegating all other events to `daypilot::update()`.

Toolkit owns global navigation. DayPilot never modifies Toolkit screen state.

## Final event flow

```text
Crossterm key
  -> DayPilot event handler
  -> DayPilotEvent
  -> Toolkit AppEvent::DayPilot
  -> Toolkit update
  -> DayPilot update or Launcher navigation
  -> Toolkit renderer
  -> DayPilot view
```

Standalone flow:

```text
Crossterm key
  -> DayPilot event handler
  -> DayPilotEvent
  -> DayPilot update
  -> DayPilot view
```

## Verification

`cargo check --workspace` passes.

The standalone executable no longer uses the old Boolean `handle_key()` interface. It also uses the shared DayPilot update and view modules.

## Preserved behavior

The refactor preserves:

- Task listing and selection
- Adding and editing tasks
- Deleting tasks and confirmation
- Completion status changes
- Form field navigation
- Priority changes
- Deadline entry
- Existing DayPilot rendering and themes
- Persistent in-memory state in Toolkit
