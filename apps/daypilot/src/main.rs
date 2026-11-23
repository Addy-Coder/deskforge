use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, time::Duration};

use daypilot::{DayPilotApp, DayPilotColors, handle_key_event, render, update};

fn main() -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    if let Err(error) = execute!(stdout, EnterAlternateScreen) {
        disable_raw_mode()?;
        return Err(error.into());
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = DayPilotApp::new();
    let colors = DayPilotColors::burnt_ember();

    loop {
        terminal.draw(|frame| {
            render(&app, frame, frame.area(), &colors);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if let Some(event) = handle_key_event(&app, key.code) {
                    if event == daypilot::DayPilotEvent::GoBack {
                        break;
                    }
                    update(&mut app, event);
                }
            }
        }
    }

    Ok(())
}
