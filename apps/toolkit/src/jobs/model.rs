use chrono::{DateTime, Local};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JobId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub id: JobId,
    pub name: String,
    pub description: String,
    pub status: JobStatus,
    pub progress: Option<u8>,
    pub started_at: Option<DateTime<Local>>,
    pub finished_at: Option<DateTime<Local>>,
    pub error: Option<String>,
}
