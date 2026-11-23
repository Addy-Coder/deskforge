use anyhow::Result;
use daypilot::{DayPilotApp, DayPilotColors};
use ratatui::style::Color;

use crate::jobs::{Job, JobContext, JobId, JobManager};

#[derive(Clone, Copy, PartialEq)]
pub enum ToolkitAction {
    DayPilot,
    WebScout,
    SysPeek,
    BackgroundJobs,
    Settings,
    Exit,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Launcher,
    DayPilot,
    Settings,
    BackgroundJobs,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    BurntEmber,
    HighContrast,
}

pub struct MenuItem {
    pub title: &'static str,
    pub description: &'static str,
    pub action: ToolkitAction,
    pub available: bool,
    pub icon: &'static str,
}

pub struct App {
    pub selected: usize,
    pub running: bool,
    pub screen: Screen,
    pub theme: Theme,
    pub menu_items: Vec<MenuItem>,
    pub daypilot: DayPilotApp,
    pub previous_screen: Screen,
    pub job_selected: usize,
    jobs: JobManager,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected: 0,
            running: true,
            screen: Screen::Launcher,
            theme: Theme::BurntEmber,
            daypilot: DayPilotApp::new(),
            previous_screen: Screen::Launcher,
            job_selected: 0,
            jobs: JobManager::new(),

            menu_items: vec![
                MenuItem {
                    title: "DayPilot",
                    description: "Daily task and project management",
                    action: ToolkitAction::DayPilot,
                    available: true,
                    icon: "◷",
                },
                MenuItem {
                    title: "WebScout",
                    description: "Web crawling and research aggregation",
                    action: ToolkitAction::WebScout,
                    available: false,
                    icon: "◎",
                },
                MenuItem {
                    title: "SysPeek",
                    description: "System resource monitoring",
                    action: ToolkitAction::SysPeek,
                    available: false,
                    icon: "▣",
                },
                MenuItem {
                    title: "Background Jobs",
                    description: "Monitor and manage background operations",
                    action: ToolkitAction::BackgroundJobs,
                    available: true,
                    icon: "◈",
                },
                MenuItem {
                    title: "Settings",
                    description: "Personal Toolkit configuration",
                    action: ToolkitAction::Settings,
                    available: true,
                    icon: "⚙",
                },
                MenuItem {
                    title: "Exit",
                    description: "Close Personal Toolkit",
                    action: ToolkitAction::Exit,
                    available: true,
                    icon: "×",
                },
            ],
        }
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.menu_items.len();
    }

    pub fn previous(&mut self) {
        self.selected = (self.selected + self.menu_items.len() - 1) % self.menu_items.len();
    }

    pub fn submit_job<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        work: F,
    ) -> JobId
    where
        F: FnOnce(JobContext) -> Result<()> + Send + 'static,
    {
        self.jobs.submit(name, description, work)
    }

    pub fn request_job_cancellation(&mut self, id: JobId) -> bool {
        self.jobs.request_cancellation(id)
    }

    pub fn process_job_messages(&mut self) {
        self.jobs.process_pending_messages();
    }

    pub fn jobs(&self) -> Vec<Job> {
        self.jobs.jobs()
    }

    pub fn job(&self, id: JobId) -> Option<Job> {
        self.jobs.job(id)
    }

    pub fn shutdown_jobs(&mut self) {
        self.jobs.shutdown();
    }
}

pub struct Palette {
    pub background: Color,
    pub panel: Color,
    pub selected_bg: Color,
    pub accent: Color,
    pub secondary_accent: Color,
    pub primary_text: Color,
    pub secondary_text: Color,
    pub muted_text: Color,
    pub border: Color,
}

impl Palette {
    pub fn from(theme: Theme) -> Self {
        match theme {
            Theme::BurntEmber => Self {
                background: Color::Rgb(9, 9, 9),
                panel: Color::Rgb(17, 17, 16),
                selected_bg: Color::Rgb(33, 24, 18),
                accent: Color::Rgb(200, 121, 77),
                secondary_accent: Color::Rgb(169, 107, 75),
                primary_text: Color::Rgb(242, 240, 237),
                secondary_text: Color::Rgb(194, 188, 182),
                muted_text: Color::Rgb(133, 129, 126),
                border: Color::Rgb(56, 43, 36),
            },

            Theme::HighContrast => Self {
                background: Color::Rgb(8, 8, 8),
                panel: Color::Rgb(17, 17, 17),
                selected_bg: Color::Rgb(36, 21, 15),
                accent: Color::Rgb(255, 118, 46),
                secondary_accent: Color::Rgb(217, 106, 50),
                primary_text: Color::White,
                secondary_text: Color::Rgb(220, 220, 220),
                muted_text: Color::Rgb(150, 150, 150),
                border: Color::Rgb(75, 43, 29),
            },
        }
    }

    pub fn daypilot_colors(&self) -> DayPilotColors {
        DayPilotColors {
            background: self.background,
            panel: self.panel,
            selected_bg: self.selected_bg,
            accent: self.accent,
            primary_text: self.primary_text,
            secondary_text: self.secondary_text,
            muted_text: self.muted_text,
            border: self.border,
        }
    }
}
