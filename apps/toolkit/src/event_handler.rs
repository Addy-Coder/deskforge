use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Screen, Theme, ToolkitAction};
use crate::events::{AppEvent, NavigationTarget};

pub fn handle_key_event(app: &App, key: KeyEvent) -> Option<AppEvent> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
        return Some(AppEvent::Exit);
    }

    if key.code == KeyCode::F(3) {
        return Some(AppEvent::OpenBackgroundJobs);
    }

    match app.screen {
        Screen::Launcher => match key.code {
            KeyCode::Down | KeyCode::Char('j') => Some(AppEvent::SelectNext),
            KeyCode::Up | KeyCode::Char('k') => Some(AppEvent::SelectPrevious),
            KeyCode::Enter => {
                let action = app.menu_items[app.selected].action;

                match action {
                    ToolkitAction::DayPilot => Some(AppEvent::Navigate(NavigationTarget::DayPilot)),
                    ToolkitAction::Settings => Some(AppEvent::Navigate(NavigationTarget::Settings)),
                    ToolkitAction::BackgroundJobs => {
                        Some(AppEvent::Navigate(NavigationTarget::BackgroundJobs))
                    }
                    ToolkitAction::Exit => Some(AppEvent::Exit),
                    _ => None,
                }
            }
            KeyCode::Char('q') => Some(AppEvent::Exit),
            _ => None,
        },

        Screen::Settings => match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Enter => {
                let theme = match app.theme {
                    Theme::BurntEmber => Theme::HighContrast,
                    Theme::HighContrast => Theme::BurntEmber,
                };
                Some(AppEvent::SetTheme(theme))
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                Some(AppEvent::Navigate(NavigationTarget::Launcher))
            }
            _ => None,
        },

        Screen::DayPilot => None,

        Screen::BackgroundJobs => match key.code {
            KeyCode::Down | KeyCode::Char('j') => Some(AppEvent::SelectNextJob),
            KeyCode::Up | KeyCode::Char('k') => Some(AppEvent::SelectPreviousJob),
            KeyCode::Char('c') | KeyCode::Char('C') => Some(AppEvent::CancelSelectedJob),
            KeyCode::Char('d') | KeyCode::Char('D') => Some(AppEvent::SubmitDemonstrationJob),
            KeyCode::Esc | KeyCode::Char('q') => Some(AppEvent::ReturnFromBackgroundJobs),
            _ => None,
        },
    }
}
