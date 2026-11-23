use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::model::{DayPilotApp, DayPilotColors, TaskStatus};

pub fn render(app: &DayPilotApp, frame: &mut Frame, area: Rect, colors: &DayPilotColors) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    render_header(app, frame, areas[0], colors);

    if app.adding_task {
        render_task_form(app, frame, areas[1], colors);
    } else if app.confirming_delete {
        render_delete_confirmation(app, frame, areas[1], colors);
    } else {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(areas[1]);

        render_tasks(app, frame, body[0], colors);
        render_details(app, frame, body[1], colors);
    }

    render_footer(app, frame, areas[2], colors);
}

fn render_header(_app: &DayPilotApp, frame: &mut Frame, area: Rect, colors: &DayPilotColors) {
    let today = Local::now().format("%A, %d %B %Y").to_string();
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " DAYPILOT ",
            Style::default()
                .fg(colors.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  ", Style::default().fg(colors.muted_text)),
        Span::styled(today, Style::default().fg(colors.primary_text)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.border))
            .style(Style::default().bg(colors.background)),
    );
    frame.render_widget(header, area);
}

fn render_tasks(app: &DayPilotApp, frame: &mut Frame, area: Rect, colors: &DayPilotColors) {
    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .map(|task| {
            let (status, status_color) = match task.status {
                TaskStatus::Pending => ("TODO", Color::Yellow),
                TaskStatus::InProgress => ("WORKING", colors.accent),
                TaskStatus::Completed => ("DONE", Color::Green),
            };
            let priority_color = match task.priority.as_str() {
                "CRITICAL" => Color::Red,
                "HIGH" => Color::LightRed,
                "MEDIUM" => Color::Yellow,
                _ => colors.muted_text,
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<9}", status),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<10}", task.priority),
                    Style::default().fg(priority_color),
                ),
                Span::styled(
                    task.title.as_str(),
                    Style::default().fg(colors.secondary_text),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" TODAY | {} TASKS ", app.tasks.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .style(Style::default().bg(colors.background)),
        )
        .highlight_style(
            Style::default()
                .bg(colors.selected_bg)
                .fg(colors.primary_text)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    let mut state = ListState::default();
    state.select(if app.tasks.is_empty() {
        None
    } else {
        Some(app.selected)
    });
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_details(app: &DayPilotApp, frame: &mut Frame, area: Rect, colors: &DayPilotColors) {
    let lines = if let Some(task) = app.tasks.get(app.selected) {
        let status = match task.status {
            TaskStatus::Pending => "Pending",
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Completed => "Completed",
        };
        vec![
            Line::from(Span::styled(
                "TASK DETAILS",
                Style::default()
                    .fg(colors.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Title: ", Style::default().fg(colors.muted_text)),
                Span::styled(&task.title, Style::default().fg(colors.primary_text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Project: ", Style::default().fg(colors.muted_text)),
                Span::styled(&task.project, Style::default().fg(colors.secondary_text)),
            ]),
            Line::from(vec![
                Span::styled("Priority: ", Style::default().fg(colors.muted_text)),
                Span::styled(&task.priority, Style::default().fg(colors.secondary_text)),
            ]),
            Line::from(vec![
                Span::styled("Deadline: ", Style::default().fg(colors.muted_text)),
                Span::styled(&task.deadline, Style::default().fg(colors.secondary_text)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(colors.muted_text)),
                Span::styled(status, Style::default().fg(colors.secondary_text)),
            ]),
        ]
    } else {
        vec![Line::from("No tasks available")]
    };

    let details = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" DETAILS ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .style(Style::default().bg(colors.panel)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(details, area);
}

fn render_task_form(app: &DayPilotApp, frame: &mut Frame, area: Rect, colors: &DayPilotColors) {
    let priorities = ["LOW", "MEDIUM", "HIGH", "CRITICAL"];
    let fields = [
        ("Task Title", app.form_title.as_str()),
        ("Project", app.form_project.as_str()),
        ("Priority", priorities[app.form_priority]),
        ("Deadline", app.form_deadline.as_str()),
    ];
    let heading = if app.editing_task {
        "EDIT EXISTING TASK"
    } else {
        "CREATE NEW TASK"
    };

    let mut lines = vec![
        Line::from(Span::styled(
            heading,
            Style::default()
                .fg(colors.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Use TAB to move between fields.",
            Style::default().fg(colors.muted_text),
        )),
        Line::from(Span::styled(
            "Use LEFT / RIGHT to change priority.",
            Style::default().fg(colors.muted_text),
        )),
        Line::from(""),
    ];

    for (index, (label, value)) in fields.iter().enumerate() {
        let focused = index == app.form_focus;
        let label_style = if focused {
            Style::default()
                .fg(colors.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.secondary_text)
        };
        let value_style = if focused {
            Style::default()
                .fg(colors.primary_text)
                .add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(colors.muted_text)
        };
        lines.push(Line::from(Span::styled(
            format!("{} {}", if focused { ">" } else { " " }, label),
            label_style,
        )));
        lines.push(Line::from(Span::styled(
            if value.is_empty() {
                "  Type here...".to_string()
            } else {
                format!("  {}", value)
            },
            value_style,
        )));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "Title is required. Empty project defaults to Personal.",
        Style::default().fg(colors.muted_text),
    )));
    let block_title = if app.editing_task {
        " EDIT TASK "
    } else {
        " ADD TASK "
    };
    let form = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .title(block_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.border))
            .style(Style::default().bg(colors.panel))
            .padding(ratatui::widgets::Padding::uniform(2)),
    );
    frame.render_widget(form, area);
}

fn render_delete_confirmation(
    app: &DayPilotApp,
    frame: &mut Frame,
    area: Rect,
    colors: &DayPilotColors,
) {
    let task_title = app
        .tasks
        .get(app.selected)
        .map(|task| task.title.as_str())
        .unwrap_or("Unknown task");
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "DELETE TASK?",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Are you sure you want to delete '{}' ?", task_title),
            Style::default().fg(colors.primary_text),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Y ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Confirm Delete     "),
            Span::styled("N / ESC ", Style::default().fg(colors.accent)),
            Span::raw("Cancel"),
        ]),
    ];
    let confirmation = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" DELETE TASK ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .style(Style::default().bg(colors.panel))
                .padding(ratatui::widgets::Padding::uniform(2)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(confirmation, area);
}

fn render_footer(app: &DayPilotApp, frame: &mut Frame, area: Rect, colors: &DayPilotColors) {
    let footer = if app.adding_task {
        Paragraph::new(Line::from(vec![
            Span::styled(" TAB ", Style::default().fg(colors.accent)),
            Span::raw("Next Field   "),
            Span::styled("ENTER ", Style::default().fg(colors.accent)),
            Span::raw("Save   "),
            Span::styled("ESC ", Style::default().fg(colors.accent)),
            Span::raw("Cancel"),
        ]))
    } else if app.confirming_delete {
        Paragraph::new(Line::from("Y Confirm Delete   N / ESC Cancel"))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(" ↑/↓ ", Style::default().fg(colors.accent)),
            Span::raw("Navigate   "),
            Span::styled("A ", Style::default().fg(colors.accent)),
            Span::raw("Add   "),
            Span::styled("E ", Style::default().fg(colors.accent)),
            Span::raw("Edit   "),
            Span::styled("D ", Style::default().fg(colors.accent)),
            Span::raw("Delete   "),
            Span::styled("SPACE ", Style::default().fg(colors.accent)),
            Span::raw("Complete   "),
            Span::styled("Q ", Style::default().fg(colors.accent)),
            Span::raw("Back"),
        ]))
    };
    frame.render_widget(footer, area);
}
