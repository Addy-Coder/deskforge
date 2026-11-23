use ratatui::style::Color;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

pub(crate) struct Task {
    pub(crate) title: String,
    pub(crate) project: String,
    pub(crate) priority: String,
    pub(crate) deadline: String,
    pub(crate) status: TaskStatus,
}

pub struct DayPilotColors {
    pub background: Color,
    pub panel: Color,
    pub selected_bg: Color,
    pub accent: Color,
    pub primary_text: Color,
    pub secondary_text: Color,
    pub muted_text: Color,
    pub border: Color,
}

impl DayPilotColors {
    pub fn burnt_ember() -> Self {
        Self {
            background: Color::Rgb(9, 9, 9),
            panel: Color::Rgb(17, 17, 16),
            selected_bg: Color::Rgb(33, 24, 18),
            accent: Color::Rgb(200, 121, 77),
            primary_text: Color::Rgb(242, 240, 237),
            secondary_text: Color::Rgb(194, 188, 182),
            muted_text: Color::Rgb(133, 129, 126),
            border: Color::Rgb(56, 43, 36),
        }
    }
}

pub struct DayPilotApp {
    pub(crate) tasks: Vec<Task>,
    pub(crate) selected: usize,
    pub(crate) adding_task: bool,
    pub(crate) editing_task: bool,
    pub(crate) form_title: String,
    pub(crate) form_project: String,
    pub(crate) form_priority: usize,
    pub(crate) form_deadline: String,
    pub(crate) form_focus: usize,
    pub(crate) confirming_delete: bool,
}

impl DayPilotApp {
    pub fn new() -> Self {
        Self {
            tasks: vec![
                Task {
                    title: "Fix worker scaling issue".into(),
                    project: "Global IDs".into(),
                    priority: "CRITICAL".into(),
                    deadline: "11:00 AM".into(),
                    status: TaskStatus::InProgress,
                },
                Task {
                    title: "Research Kafka partitioning".into(),
                    project: "Learning".into(),
                    priority: "HIGH".into(),
                    deadline: "02:00 PM".into(),
                    status: TaskStatus::Pending,
                },
                Task {
                    title: "Update project documentation".into(),
                    project: "Personal".into(),
                    priority: "LOW".into(),
                    deadline: "05:00 PM".into(),
                    status: TaskStatus::Pending,
                },
                Task {
                    title: "Review pull requests".into(),
                    project: "Global IDs".into(),
                    priority: "MEDIUM".into(),
                    deadline: "06:00 PM".into(),
                    status: TaskStatus::Pending,
                },
            ],
            selected: 0,
            adding_task: false,
            editing_task: false,
            form_title: String::new(),
            form_project: String::new(),
            form_priority: 1,
            form_deadline: String::new(),
            form_focus: 0,
            confirming_delete: false,
        }
    }
}
