# Stage 2: Event Handling and Rendering Separation

## Summary

Stage 2 separates keyboard conversion and UI rendering from `apps/toolkit/src/main.rs`.

The active application flow is:

```text
Crossterm Input
  -> event_handler.rs
  -> AppEvent
  -> update.rs
  -> App State
  -> renderer.rs
  -> Terminal UI
```

## Changed files

### `apps/toolkit/src/event_handler.rs`

Created `handle_key_event(app: &App, key: KeyEvent) -> Option<AppEvent>`.

It converts:

- Launcher navigation keys into selection events.
- Enter selection into DayPilot, Settings, or Exit events.
- Settings navigation and theme keys into navigation or theme events.
- `Ctrl+Q` into `AppEvent::Exit` globally.
- `Q` in Launcher or Settings into the existing exit/back behavior.

The handler identifies requests but does not mutate `App` state.

### `apps/toolkit/src/renderer.rs`

Created `pub fn render(frame: &mut Frame, app: &App)`.

This module now owns:

- Main screen layout.
- Header and footer rendering.
- Launcher menu rendering.
- Application details rendering.
- Settings rendering.
- Theme-dependent palettes and existing visual styles.
- DayPilot delegation through `DayPilotApp::render()`.

Rendering reads application state and does not modify it or process keyboard input.

### `apps/toolkit/src/main.rs`

Reduced to runtime orchestration:

- Terminal initialization.
- Terminal restoration.
- Application construction.
- Crossterm polling.
- Pressed-key filtering.
- Calling `event_handler::handle_key_event()`.
- Calling `update::update()`.
- Calling `renderer::render()`.

The existing DayPilot `handle_key()` interface remains in `main.rs` for this stage.

### Existing files

- `apps/toolkit/src/app.rs` remains the application state and palette source.
- `apps/toolkit/src/events.rs` remains the typed Toolkit event source.
- `apps/toolkit/src/update.rs` remains the centralized application state transition function.

## Keyboard flow

### Launcher

```text
Crossterm key
  -> event_handler::handle_key_event()
  -> AppEvent::SelectNext / SelectPrevious
  -> update::update()
  -> App::next() / App::previous()
  -> renderer::render()
```

Enter is translated according to the selected launcher action:

- DayPilot -> `Navigate(DayPilot)`
- Settings -> `Navigate(Settings)`
- Exit -> `Exit`

### Settings

- Left, Right, or Enter -> `SetTheme(...)`
- Escape or Q -> `Navigate(Launcher)`

### DayPilot

DayPilot continues to use `handle_key(key.code)`. If it requests navigation back to Toolkit, `main.rs` creates:

```text
AppEvent::Navigate(NavigationTarget::Launcher)
```

and passes it to `update()`.

## Main responsibilities

`main.rs` now remains responsible for:

- Terminal setup and alternate-screen behavior.
- Terminal restoration on exit.
- The Crossterm polling loop.
- Delegating Toolkit keys.
- Delegating existing DayPilot keys.
- Ordering updates and redraws.

It no longer contains the application renderer or direct screen/theme state mutations.

## Verification

`cargo check --workspace` passes.

The remaining warning is unrelated to this stage: the standalone DayPilot executable imports `KeyCode` without using it.

## Remaining architectural limitations

- DayPilot keyboard handling and task state mutation remain combined in `DayPilotApp::handle_key()`.
- DayPilot navigation still uses a Boolean return value.
- DayPilot rendering and business logic remain in `apps/daypilot/src/lib.rs`.
- The legacy renderer block may be removed in a subsequent cleanup.
- Background jobs are not implemented.
