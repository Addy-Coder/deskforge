use crate::app::Theme;
use daypilot::DayPilotEvent;

#[derive(Clone, Copy, PartialEq)]
pub enum NavigationTarget {
    Launcher,
    DayPilot,
    Settings,
    BackgroundJobs,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AppEvent {
    Navigate(NavigationTarget),
    Exit,
    SetTheme(Theme),
    SelectNext,
    SelectPrevious,
    DayPilot(DayPilotEvent),
    OpenBackgroundJobs,
    ReturnFromBackgroundJobs,
    SelectNextJob,
    SelectPreviousJob,
    CancelSelectedJob,
    SubmitDemonstrationJob,
}
