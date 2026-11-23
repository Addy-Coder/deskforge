# Stage 1: Toolkit Event and Update Architecture

## Completed changes

### `apps/toolkit/src/events.rs`

Added strongly typed application events:

- `NavigationTarget`
  - `Launcher`
  - `DayPilot`
  - `Settings`
- `AppEvent`
  - `Navigate(NavigationTarget)`
  - `Exit`
  - `SetTheme(Theme)`
  - `SelectNext`
  - `SelectPrevious`

These enums replace direct screen and running-state mutations for the launcher and settings flows.

### `apps/toolkit/src/update.rs`

Added:

```rust
pub fn update(app: &mut App, event: AppEvent)
```

This function centrally handles:

- Navigation between Launcher, DayPilot, and Settings.
- Application exit.
- Theme changes.
- Launcher menu selection movement.

The update function does not render or read terminal input.

### `apps/toolkit/src/main.rs`

Updated `run_app()` so launcher and settings keyboard actions create `AppEvent` values and pass them to `update()`.

DayPilot remains connected to its existing `handle_key()` interface. Its Boolean result is translated into a typed `Navigate(Launcher)` event.

### `apps/toolkit/src/app.rs`

Removed the unused `toggle_theme()` method because theme transitions are now handled by `AppEvent::SetTheme` and `update()`.

## Event flow

```text
Crossterm KeyEvent
  -> run_app()
  -> AppEvent
  -> update(app, event)
  -> App state mutation
  -> existing renderer
```

Examples:

```text
Down key
  -> AppEvent::SelectNext
  -> app.next()
```

```text
Enter on Settings
  -> AppEvent::Navigate(NavigationTarget::Settings)
  -> app.screen = Screen::Settings
```

```text
Left, Right, or Enter in Settings
  -> AppEvent::SetTheme(Theme)
  -> app.theme = theme
```

```text
Esc or Q in Settings
  -> AppEvent::Navigate(NavigationTarget::Launcher)
  -> app.screen = Screen::Launcher
```

## Responsibilities still in `main.rs`

`main.rs` still owns:

- Terminal initialization and restoration.
- The Crossterm event loop.
- Keyboard-to-event conversion.
- Calling `update()`.
- Rendering and UI layout.
- The existing call to `DayPilotApp::handle_key()`.

## Verification

`cargo check --workspace` passes.

`AppEvent` and `NavigationTarget` are now genuinely used in Toolkit application code.

The only remaining workspace warning is in the standalone DayPilot executable: `KeyCode` is imported but unused.

## Next stage

Stage 2 should extract:

- Event handling into `event_handler.rs`.
- Rendering into `renderer.rs`.
- Terminal ownership into `runtime/terminal.rs`, if implemented without unnecessary abstraction.

DayPilot module refactoring should remain separate and is reserved for Stage 4.
