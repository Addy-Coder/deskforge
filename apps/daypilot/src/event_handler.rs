use crossterm::event::KeyCode;

use crate::{DayPilotApp, DayPilotEvent};

pub fn handle_key_event(app: &DayPilotApp, key: KeyCode) -> Option<DayPilotEvent> {
    if app.confirming_delete {
        return match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(DayPilotEvent::ConfirmDelete),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                Some(DayPilotEvent::CancelDelete)
            }
            _ => None,
        };
    }

    if app.adding_task {
        return match key {
            KeyCode::Esc => Some(DayPilotEvent::CancelForm),
            KeyCode::Tab => Some(DayPilotEvent::MoveFormFieldNext),
            KeyCode::BackTab => Some(DayPilotEvent::MoveFormFieldPrevious),
            KeyCode::Enter => Some(DayPilotEvent::SaveTask),
            KeyCode::Left if app.form_focus == 2 => Some(DayPilotEvent::ChangePriorityPrevious),
            KeyCode::Right if app.form_focus == 2 => Some(DayPilotEvent::ChangePriorityNext),
            KeyCode::Char(c) if !c.is_control() => Some(DayPilotEvent::InsertText(c)),
            KeyCode::Backspace => Some(DayPilotEvent::DeletePreviousCharacter),
            _ => None,
        };
    }

    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(DayPilotEvent::GoBack),
        KeyCode::Char('a') | KeyCode::Char('A') => Some(DayPilotEvent::StartAddingTask),
        KeyCode::Char('e') | KeyCode::Char('E') => Some(DayPilotEvent::StartEditingTask),
        KeyCode::Char('d') | KeyCode::Char('D') => Some(DayPilotEvent::StartDeletingTask),
        KeyCode::Down | KeyCode::Char('j') => Some(DayPilotEvent::MoveSelectionDown),
        KeyCode::Up | KeyCode::Char('k') => Some(DayPilotEvent::MoveSelectionUp),
        KeyCode::Char(' ') => Some(DayPilotEvent::ToggleCompletion),
        _ => None,
    }
}
