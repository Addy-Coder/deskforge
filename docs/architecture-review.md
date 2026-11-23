# Personal Toolkit Architecture Review

## Conclusion

The current refactor is only a partial state extraction. The application is not event-driven yet.

`AppEvent` and `NavigationTarget` are disconnected declarations:

- `AppEvent` is never imported or matched.
- `NavigationTarget` is only referenced by `AppEvent`.
- There is no `update()` function.
- There is no event handler module.
- Keyboard handling directly mutates `App` or calls `DayPilotApp::handle_key()`.

## 1. Crossterm input and the complete current flow

Crossterm input enters through:

- `apps/toolkit/src/main.rs`
- `main()`
- `run_app()`

`run_app()` directly calls:

- `event::poll(...)`
- `event::read()`
- `Event::Key(key)`
- `KeyEventKind::Press`

No Crossterm event becomes an application event.

The same function directly dispatches input based on the active screen:

- Launcher: match `key.code`
- DayPilot: `app.daypilot.handle_key(key.code)`
- Settings: match `key.code`

This mixes keyboard conversion, state mutation, and application orchestration in the main event loop.

## 2. Keyboard-to-event conversion

There is currently no conversion function.

Keyboard matching happens directly inside `run_app()`:

- Launcher keys call `App::next()`, `App::previous()`, or change `app.screen`.
- Exit keys change `app.running`.
- Settings keys call `App::toggle_theme()` or change `app.screen`.
- DayPilot keys are passed into `DayPilotApp::handle_key()`.

## 3. `AppEvent` handling

Nowhere in the workspace.

`apps/toolkit/src/events.rs` defines:

- `NavigationTarget`
- `AppEvent`

However, no source file imports or matches either enum.

`NavigationTarget` and `SetTheme(Theme)` currently have no execution path.

## 4. `update()` function

There is no Toolkit `update()` function.

Global state transitions are performed directly inside `run_app()`:

```rust
app.next();
app.previous();
app.screen = Screen::DayPilot;
app.screen = Screen::Settings;
app.running = false;
app.toggle_theme();
```

## 5. Updating `App`

Direct mutable access occurs in `run_app()`.

For example:

- Launcher navigation calls `App::next()` or `App::previous()`.
- Enter changes `app.screen`.
- Exit changes `app.running`.
- Settings changes `app.theme` through `App::toggle_theme()`.

`App::next()`, `previous()`, and `toggle_theme()` are in `apps/toolkit/src/app.rs`, but they are model methods rather than part of a centralized update layer.

## 6. Rendering

Redraws happen in `run_app()`:

```rust
terminal.draw(|frame| render(frame, &app))?;
```

The `render()` function is in `apps/toolkit/src/main.rs`. It:

- Reads `app.theme`.
- Inspects `app.screen`.
- Dispatches to `render_launcher()` or `render_settings()`.
- Calls `app.daypilot.render()` for DayPilot.

The renderer is not separated from `main.rs`, although it already mostly reads state and does not modify it.

DayPilot rendering is in `apps/daypilot/src/lib.rs`, primarily in `DayPilotApp::render()` and its private view helpers.

## 7. DayPilot navigation communication

DayPilot communicates navigation through the Boolean return value from:

```rust
pub fn handle_key(&mut self, key: KeyCode) -> bool
```

This function is in `apps/daypilot/src/lib.rs`.

In `run_app()`:

```rust
if app.daypilot.handle_key(key.code) {
    app.screen = Screen::Launcher;
}
```

Problems with this interface:

- `true` means “go back.”
- `false` means “keep handling the key.”
- The meaning of navigation is implied by a Boolean.
- DayPilot keyboard handling also directly mutates task and form state.

Examples include:

- `KeyCode::Char('a') => self.start_adding_task()`
- `KeyCode::Char(' ') => self.toggle_completed()`
- `KeyCode::Enter => self.save_task()`

## Disconnected components

| Component                  | Status                               |
| -------------------------- | ------------------------------------ |
| `app.rs`                   | Implemented and wired into `main.rs` |
| `events.rs`                | Declared but unused                  |
| Event handler              | Missing                              |
| `update()`                 | Missing                              |
| Separate renderer          | Missing                              |
| Typed DayPilot events      | Missing                              |
| DayPilot update layer      | Missing                              |
| Background job preparation | Missing                              |

## Duplicated or inconsistent responsibilities

- `run_app()` owns keyboard dispatch, navigation, and state mutation.
- `App` methods mutate state but are called directly from `run_app()`.
- DayPilot owns both keyboard interpretation and state mutation.
- DayPilot combines model, update logic, and rendering in `lib.rs`.
- Toolkit rendering and terminal orchestration remain together in `main.rs`.
- The standalone DayPilot executable owns a separate terminal loop and calls `handle_key()` directly.

## Recommended smallest changes

### 1. Add an application update layer

Create `apps/toolkit/src/update.rs`:

```rust
pub fn update(app: &mut App, event: AppEvent) {
    // Handle navigation, exit, theme changes, and module events.
}
```

The update function should handle:

- Navigation
- Exit
- Theme changes
- Delegation of DayPilot events

### 2. Add an event adapter

Create `apps/toolkit/src/event_handler.rs`.

It should:

- Convert `KeyEvent` into Toolkit or module events.
- Route input based on `app.screen`.
- Handle global shortcuts such as `Ctrl+Q`.
- Delegate module key interpretation to DayPilot.

The target flow should be:

```text
Crossterm KeyEvent
  -> event_handler
  -> AppEvent
  -> update()
  -> App mutation
  -> redraw
```

### 3. Replace the DayPilot Boolean interface

Create:

- `apps/daypilot/src/events.rs`
- `apps/daypilot/src/update.rs`

Define meaningful events such as:

```rust
pub enum DayPilotEvent {
    MoveSelectionUp,
    MoveSelectionDown,
    StartAdding,
    StartEditing,
    StartDeleting,
    ToggleCompletion,
    SaveTask,
    CancelForm,
    GoBack,
}
```

DayPilot should expose:

```rust
pub fn event_from_key(&self, key: KeyCode) -> Option<DayPilotEvent>
pub fn update(&mut self, event: DayPilotEvent)
```

The update function should contain the task business logic currently spread across private methods.

### 4. Connect DayPilot events to Toolkit events

Add a DayPilot variant to `AppEvent`:

```rust
pub enum AppEvent {
    Navigate(NavigationTarget),
    Exit,
    ChangeTheme,
    SetTheme(Theme),
    DayPilot(DayPilotEvent),
}
```

Toolkit’s `update()` can delegate the event:

```rust
app.daypilot.update(daypilot_event);
```

Navigation from DayPilot can produce:

```rust
AppEvent::Navigate(NavigationTarget::Launcher)
```

### 5. Extract Toolkit rendering

Move `render()` and its rendering helpers from `main.rs` into `renderer.rs`.

Keep `main.rs` focused on:

- Terminal initialization
- Application construction
- Event polling
- Calling `update()`
- Calling `render()`
