use crate::model::{Task, TaskStatus};
use crate::{DayPilotApp, DayPilotEvent};

pub fn update(app: &mut DayPilotApp, event: DayPilotEvent) {
    match event {
        DayPilotEvent::MoveSelectionUp => move_previous(app),
        DayPilotEvent::MoveSelectionDown => move_next(app),
        DayPilotEvent::ToggleCompletion => toggle_completed(app),
        DayPilotEvent::StartAddingTask => start_adding_task(app),
        DayPilotEvent::StartEditingTask => start_editing_task(app),
        DayPilotEvent::StartDeletingTask => start_deleting_task(app),
        DayPilotEvent::ConfirmDelete => delete_selected_task(app),
        DayPilotEvent::CancelDelete => app.confirming_delete = false,
        DayPilotEvent::SaveTask => save_task(app),
        DayPilotEvent::CancelForm => cancel_form(app),
        DayPilotEvent::MoveFormFieldNext => {
            app.form_focus = (app.form_focus + 1) % 4;
        }
        DayPilotEvent::MoveFormFieldPrevious => {
            app.form_focus = (app.form_focus + 3) % 4;
        }
        DayPilotEvent::ChangePriorityNext => {
            if app.form_focus == 2 {
                app.form_priority = (app.form_priority + 1) % 4;
            }
        }
        DayPilotEvent::ChangePriorityPrevious => {
            if app.form_focus == 2 {
                app.form_priority = (app.form_priority + 3) % 4;
            }
        }
        DayPilotEvent::InsertText(c) => insert_text(app, c),
        DayPilotEvent::DeletePreviousCharacter => delete_previous_character(app),
        DayPilotEvent::GoBack => {}
    }
}

fn move_next(app: &mut DayPilotApp) {
    if !app.tasks.is_empty() {
        app.selected = (app.selected + 1) % app.tasks.len();
    }
}

fn move_previous(app: &mut DayPilotApp) {
    if !app.tasks.is_empty() {
        app.selected = (app.selected + app.tasks.len() - 1) % app.tasks.len();
    }
}

fn toggle_completed(app: &mut DayPilotApp) {
    if let Some(task) = app.tasks.get_mut(app.selected) {
        task.status = if task.status == TaskStatus::Completed {
            TaskStatus::Pending
        } else {
            TaskStatus::Completed
        };
    }
}

fn start_adding_task(app: &mut DayPilotApp) {
    app.adding_task = true;
    app.editing_task = false;
    app.form_title.clear();
    app.form_project.clear();
    app.form_priority = 1;
    app.form_deadline.clear();
    app.form_focus = 0;
}

fn start_editing_task(app: &mut DayPilotApp) {
    if let Some(task) = app.tasks.get(app.selected) {
        app.form_title = task.title.clone();
        app.form_project = task.project.clone();
        app.form_deadline = task.deadline.clone();
        app.form_priority = match task.priority.as_str() {
            "LOW" => 0,
            "MEDIUM" => 1,
            "HIGH" => 2,
            "CRITICAL" => 3,
            _ => 1,
        };
        app.form_focus = 0;
        app.adding_task = true;
        app.editing_task = true;
    }
}

fn save_task(app: &mut DayPilotApp) {
    let title = app.form_title.trim().to_string();
    if title.is_empty() {
        return;
    }

    let project = if app.form_project.trim().is_empty() {
        "Personal".to_string()
    } else {
        app.form_project.trim().to_string()
    };

    let priorities = ["LOW", "MEDIUM", "HIGH", "CRITICAL"];
    let priority = priorities[app.form_priority].to_string();
    let deadline = if app.form_deadline.trim().is_empty() {
        "No deadline".to_string()
    } else {
        app.form_deadline.trim().to_string()
    };

    if app.editing_task {
        if let Some(task) = app.tasks.get_mut(app.selected) {
            task.title = title;
            task.project = project;
            task.priority = priority;
            task.deadline = deadline;
        }
    } else {
        app.tasks.push(Task {
            title,
            project,
            priority,
            deadline,
            status: TaskStatus::Pending,
        });
        app.selected = app.tasks.len() - 1;
    }

    app.adding_task = false;
    app.editing_task = false;
}

fn cancel_form(app: &mut DayPilotApp) {
    app.adding_task = false;
    app.editing_task = false;
}

fn start_deleting_task(app: &mut DayPilotApp) {
    if !app.tasks.is_empty() {
        app.confirming_delete = true;
    }
}

fn delete_selected_task(app: &mut DayPilotApp) {
    if app.tasks.is_empty() {
        app.confirming_delete = false;
        return;
    }

    app.tasks.remove(app.selected);

    if app.tasks.is_empty() {
        app.selected = 0;
    } else if app.selected >= app.tasks.len() {
        app.selected = app.tasks.len() - 1;
    }

    app.confirming_delete = false;
}

fn insert_text(app: &mut DayPilotApp, c: char) {
    match app.form_focus {
        0 => app.form_title.push(c),
        1 => app.form_project.push(c),
        3 => app.form_deadline.push(c),
        _ => {}
    }
}

fn delete_previous_character(app: &mut DayPilotApp) {
    match app.form_focus {
        0 => {
            app.form_title.pop();
        }
        1 => {
            app.form_project.pop();
        }
        3 => {
            app.form_deadline.pop();
        }
        _ => {}
    }
}
