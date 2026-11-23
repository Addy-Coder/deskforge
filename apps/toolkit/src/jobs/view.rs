use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use super::{Job, JobStatus};
use crate::app::Palette;

pub fn render(frame: &mut Frame, area: Rect, jobs: &[Job], selected: usize, palette: &Palette) {
    let selected = if jobs.is_empty() {
        0
    } else {
        selected.min(jobs.len() - 1)
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(area);

    render_header(frame, layout[0], jobs.len(), palette);
    render_summary(frame, layout[1], jobs, palette);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(layout[2]);

    render_job_list(frame, body[0], jobs, selected, palette);
    render_details(frame, body[1], jobs.get(selected), palette);
    render_footer(frame, layout[3], palette);
}

fn render_header(frame: &mut Frame, area: Rect, total: usize, palette: &Palette) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " BACKGROUND JOBS ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("| {} JOB{}", total, if total == 1 { "" } else { "S" }),
            Style::default().fg(palette.primary_text),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)),
    );
    frame.render_widget(header, area);
}

fn render_summary(frame: &mut Frame, area: Rect, jobs: &[Job], palette: &Palette) {
    let mut running = 0;
    let mut queued = 0;
    let mut completed = 0;
    let mut failed = 0;
    let mut cancelled = 0;

    for job in jobs {
        match job.status {
            JobStatus::Running => running += 1,
            JobStatus::Queued => queued += 1,
            JobStatus::Completed => completed += 1,
            JobStatus::Failed => failed += 1,
            JobStatus::Cancelled => cancelled += 1,
        }
    }

    let summary = Paragraph::new(Line::from(vec![
        summary_span("RUNNING", running, palette.accent),
        Span::raw("  "),
        summary_span("QUEUED", queued, palette.secondary_text),
        Span::raw("  "),
        summary_span("COMPLETED", completed, Color::Green),
        Span::raw("  "),
        summary_span("FAILED", failed, Color::Red),
        Span::raw("  "),
        summary_span("CANCELLED", cancelled, palette.muted_text),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(palette.border)),
    );
    frame.render_widget(summary, area);
}

fn summary_span(label: &str, count: usize, color: Color) -> Span<'static> {
    Span::styled(
        format!("{} {}", label, count),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn status_style(status: JobStatus, palette: &Palette) -> (String, Color) {
    match status {
        JobStatus::Queued => ("QUEUED".into(), palette.secondary_text),
        JobStatus::Running => ("RUNNING".into(), palette.accent),
        JobStatus::Completed => ("COMPLETED".into(), Color::Green),
        JobStatus::Failed => ("FAILED".into(), Color::Red),
        JobStatus::Cancelled => ("CANCELLED".into(), palette.muted_text),
    }
}

fn render_job_list(
    frame: &mut Frame,
    area: Rect,
    jobs: &[Job],
    selected: usize,
    palette: &Palette,
) {
    let items: Vec<ListItem> = jobs
        .iter()
        .map(|job| {
            let (status_text, status_color) = status_style(job.status, palette);
            let progress = job
                .progress
                .map(|value| format!(" {}%", value))
                .unwrap_or_else(|| " --".into());

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!("#{} ", job.id.0),
                        Style::default().fg(palette.muted_text),
                    ),
                    Span::styled(
                        job.name.as_str(),
                        Style::default()
                            .fg(palette.primary_text)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!("  {:<10}", status_text),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(progress, Style::default().fg(palette.secondary_text)),
                ]),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" JOBS ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.border)),
        )
        .highlight_style(
            Style::default()
                .bg(palette.selected_bg)
                .fg(palette.primary_text)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    let mut state = ListState::default();
    state.select(if jobs.is_empty() {
        None
    } else {
        Some(selected)
    });
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_details(frame: &mut Frame, area: Rect, job: Option<&Job>, palette: &Palette) {
    let lines = if let Some(job) = job {
        let (status_text, status_color) = status_style(job.status, palette);
        let job_id = job.id.0.to_string();
        let progress = job
            .progress
            .map(|value| format!("{}%", value))
            .unwrap_or_else(|| "Unavailable".into());
        let started = format_timestamp(job.started_at);
        let finished = format_timestamp(job.finished_at);
        vec![
            Line::from(Span::styled(
                "SELECTED JOB",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            detail_line("Name", &job.name, palette.primary_text, palette),
            detail_line(
                "Description",
                &job.description,
                palette.secondary_text,
                palette,
            ),
            detail_line("Job ID", &job_id, palette.secondary_text, palette),
            detail_line("Status", &status_text, status_color, palette),
            detail_line("Progress", &progress, palette.secondary_text, palette),
            detail_line("Started", &started, palette.secondary_text, palette),
            detail_line("Finished", &finished, palette.secondary_text, palette),
            detail_line(
                "Error",
                job.error.as_deref().unwrap_or("None"),
                if job.error.is_some() {
                    Color::Red
                } else {
                    palette.muted_text
                },
                palette,
            ),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                "SELECTED JOB",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "No jobs registered.",
                Style::default().fg(palette.muted_text),
            )),
            Line::from(Span::styled(
                "Press D to submit a temporary demonstration job.",
                Style::default().fg(palette.secondary_text),
            )),
        ]
    };

    let details = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .title(" DETAILS ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)),
    );
    frame.render_widget(details, area);
}

fn detail_line(label: &str, value: &str, value_color: Color, palette: &Palette) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{}: ", label),
            Style::default().fg(palette.muted_text),
        ),
        Span::styled(value.to_owned(), Style::default().fg(value_color)),
    ])
}

fn format_timestamp(timestamp: Option<chrono::DateTime<chrono::Local>>) -> String {
    timestamp
        .map(|value| value.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "Not available".into())
}

fn render_footer(frame: &mut Frame, area: Rect, palette: &Palette) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓ ", Style::default().fg(palette.accent)),
        Span::raw("Navigate   "),
        Span::styled("C ", Style::default().fg(palette.accent)),
        Span::raw("Cancel   "),
        Span::styled("D ", Style::default().fg(palette.accent)),
        Span::raw("Demo   "),
        Span::styled("F3 ", Style::default().fg(palette.accent)),
        Span::raw("Toggle   "),
        Span::styled("Q/ESC ", Style::default().fg(palette.accent)),
        Span::raw("Back"),
    ]))
    .alignment(ratatui::layout::Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(palette.border)),
    );
    frame.render_widget(footer, area);
}
