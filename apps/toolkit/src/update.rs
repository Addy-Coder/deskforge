use std::thread;
use std::time::Duration;

use crate::app::{App, Screen};
use crate::events::{AppEvent, NavigationTarget};
use crate::jobs::JobStatus;

pub fn update(app: &mut App, event: AppEvent) {
    match event {
        AppEvent::Navigate(target) => match target {
            NavigationTarget::Launcher => app.screen = Screen::Launcher,
            NavigationTarget::DayPilot => app.screen = Screen::DayPilot,
            NavigationTarget::Settings => app.screen = Screen::Settings,
            NavigationTarget::BackgroundJobs => {
                if app.screen != Screen::BackgroundJobs {
                    app.previous_screen = app.screen;
                    app.screen = Screen::BackgroundJobs;
                }
            }
        },
        AppEvent::Exit => app.running = false,
        AppEvent::SetTheme(theme) => app.theme = theme,
        AppEvent::SelectNext => app.next(),
        AppEvent::SelectPrevious => app.previous(),
        AppEvent::DayPilot(event) => {
            if event == daypilot::DayPilotEvent::GoBack {
                app.screen = Screen::Launcher;
            } else {
                daypilot::update(&mut app.daypilot, event);
            }
        }
        AppEvent::OpenBackgroundJobs => {
            if app.screen != Screen::BackgroundJobs {
                app.previous_screen = app.screen;
                app.screen = Screen::BackgroundJobs;
            }
        }
        AppEvent::ReturnFromBackgroundJobs => {
            app.screen = app.previous_screen;
        }
        AppEvent::SelectNextJob => {
            let count = app.jobs().len();
            if count > 0 {
                app.job_selected = (app.job_selected + 1) % count;
            }
        }
        AppEvent::SelectPreviousJob => {
            let count = app.jobs().len();
            if count > 0 {
                app.job_selected = (app.job_selected + count - 1) % count;
            }
        }
        AppEvent::CancelSelectedJob => {
            if let Some(job) = app.jobs().get(app.job_selected).cloned() {
                if matches!(job.status, JobStatus::Queued | JobStatus::Running) {
                    app.request_job_cancellation(job.id);
                }
            }
        }
        AppEvent::SubmitDemonstrationJob => {
            app.submit_job(
                "Temporary Demo Job",
                "Testing demonstration: reports progress and completes after approximately 12 seconds.",
                |context| {
                    for progress in 0..=100 {
                        if context.is_cancelled() {
                            return Ok(());
                        }
                        context.report_progress(progress);
                        thread::sleep(Duration::from_millis(120));
                    }
                    Ok(())
                },
            );
        }
    }
}
