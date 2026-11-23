use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};

use anyhow::Result;
use chrono::Local;

use super::model::{Job, JobId, JobStatus};

#[derive(Clone, Debug)]
pub struct JobContext {
    job_id: JobId,
    event_tx: mpsc::Sender<WorkerMessage>,
    cancelled: Arc<AtomicBool>,
}

impl JobContext {
    pub fn report_progress(&self, progress: u8) {
        let progress = progress.min(100);
        let _ = self.event_tx.send(WorkerMessage::Progress {
            job_id: self.job_id,
            progress,
        });
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

enum WorkerMessage {
    Started { job_id: JobId },
    Progress { job_id: JobId, progress: u8 },
    Completed { job_id: JobId },
    Failed { job_id: JobId, error: String },
    Cancelled { job_id: JobId },
}

pub struct JobManager {
    jobs: HashMap<JobId, Job>,
    cancellation: HashMap<JobId, Arc<AtomicBool>>,
    workers: HashMap<JobId, JoinHandle<()>>,
    next_id: u64,
    event_tx: mpsc::Sender<WorkerMessage>,
    event_rx: mpsc::Receiver<WorkerMessage>,
    shutdown_started: bool,
}

impl JobManager {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            jobs: HashMap::new(),
            cancellation: HashMap::new(),
            workers: HashMap::new(),
            next_id: 1,
            event_tx,
            event_rx,
            shutdown_started: false,
        }
    }

    pub fn submit<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        work: F,
    ) -> JobId
    where
        F: FnOnce(JobContext) -> Result<()> + Send + 'static,
    {
        let id = JobId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);

        self.jobs.insert(
            id,
            Job {
                id,
                name: name.into(),
                description: description.into(),
                status: JobStatus::Queued,
                progress: None,
                started_at: None,
                finished_at: None,
                error: None,
            },
        );

        let cancelled = Arc::new(AtomicBool::new(false));
        self.cancellation.insert(id, Arc::clone(&cancelled));

        let event_tx = self.event_tx.clone();
        let context = JobContext {
            job_id: id,
            event_tx: event_tx.clone(),
            cancelled: Arc::clone(&cancelled),
        };

        let handle = thread::spawn(move || {
            let _ = event_tx.send(WorkerMessage::Started { job_id: id });

            if context.is_cancelled() {
                let _ = event_tx.send(WorkerMessage::Cancelled { job_id: id });
                return;
            }

            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| work(context.clone())));

            if context.is_cancelled() {
                let _ = event_tx.send(WorkerMessage::Cancelled { job_id: id });
                return;
            }

            match result {
                Ok(Ok(())) => {
                    let _ = event_tx.send(WorkerMessage::Completed { job_id: id });
                }
                Ok(Err(error)) => {
                    let _ = event_tx.send(WorkerMessage::Failed {
                        job_id: id,
                        error: error.to_string(),
                    });
                }
                Err(_) => {
                    let _ = event_tx.send(WorkerMessage::Failed {
                        job_id: id,
                        error: "Job worker panicked".to_string(),
                    });
                }
            }
        });

        self.workers.insert(id, handle);
        id
    }

    pub fn request_cancellation(&mut self, id: JobId) -> bool {
        let Some(job) = self.jobs.get(&id) else {
            return false;
        };

        if job.status != JobStatus::Queued && job.status != JobStatus::Running {
            return false;
        }

        if let Some(cancelled) = self.cancellation.get(&id) {
            cancelled.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

    pub fn process_pending_messages(&mut self) {
        while let Ok(message) = self.event_rx.try_recv() {
            self.process_message(message);
        }
    }

    pub fn jobs(&self) -> Vec<Job> {
        let mut jobs: Vec<Job> = self.jobs.values().cloned().collect();
        jobs.sort_by_key(|job| job.id.0);
        jobs
    }

    pub fn job(&self, id: JobId) -> Option<Job> {
        self.jobs.get(&id).cloned()
    }

    pub fn shutdown(&mut self) {
        if self.shutdown_started {
            return;
        }
        self.shutdown_started = true;

        let ids: Vec<JobId> = self.jobs.keys().copied().collect();
        for id in ids {
            self.request_cancellation(id);
        }

        self.process_pending_messages();

        let handles: Vec<(JobId, JoinHandle<()>)> = self
            .workers
            .drain()
            .map(|(id, handle)| (id, handle))
            .collect();

        for (id, handle) in handles {
            let _ = handle.join();
            self.process_pending_messages();

            if let Some(job) = self.jobs.get_mut(&id) {
                if job.status == JobStatus::Queued || job.status == JobStatus::Running {
                    job.status = JobStatus::Cancelled;
                    job.finished_at = Some(Local::now());
                }
            }
        }
    }

    fn process_message(&mut self, message: WorkerMessage) {
        match message {
            WorkerMessage::Started { job_id } => {
                if let Some(job) = self.jobs.get_mut(&job_id) {
                    if job.status == JobStatus::Queued {
                        job.status = JobStatus::Running;
                        job.started_at = Some(Local::now());
                    }
                }
            }
            WorkerMessage::Progress { job_id, progress } => {
                if let Some(job) = self.jobs.get_mut(&job_id) {
                    if job.status == JobStatus::Running {
                        job.progress = Some(progress.min(100));
                    }
                }
            }
            WorkerMessage::Completed { job_id } => {
                if let Some(job) = self.jobs.get_mut(&job_id) {
                    if job.status == JobStatus::Running {
                        job.status = JobStatus::Completed;
                        job.finished_at = Some(Local::now());
                        if job.progress.is_none() {
                            job.progress = Some(100);
                        } else {
                            job.progress = Some(job.progress.unwrap_or(0).min(100));
                        }
                        job.error = None;
                    }
                }
            }
            WorkerMessage::Failed { job_id, error } => {
                if let Some(job) = self.jobs.get_mut(&job_id) {
                    if job.status == JobStatus::Running {
                        job.status = JobStatus::Failed;
                        job.finished_at = Some(Local::now());
                        job.error = Some(error);
                    }
                }
            }
            WorkerMessage::Cancelled { job_id } => {
                if let Some(job) = self.jobs.get_mut(&job_id) {
                    if job.status == JobStatus::Queued || job.status == JobStatus::Running {
                        job.status = JobStatus::Cancelled;
                        job.finished_at = Some(Local::now());
                    }
                }
            }
        }
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier, Mutex, mpsc as std_mpsc};
    use std::time::{Duration, Instant};

    fn wait_for_job<F>(manager: &mut JobManager, id: JobId, predicate: F)
    where
        F: Fn(&Job) -> bool,
    {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            manager.process_pending_messages();
            if manager.job(id).is_some_and(|job| predicate(&job)) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for job {id:?}"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn submit_returns_unique_job_id() {
        let mut manager = JobManager::new();
        let first = manager.submit("first", "test", |_| Ok(()));
        let second = manager.submit("second", "test", |_| Ok(()));

        assert_ne!(first, second);
        assert_eq!(manager.job(first).unwrap().status, JobStatus::Queued);
        assert_eq!(manager.job(second).unwrap().status, JobStatus::Queued);
    }

    #[test]
    fn successful_job_completes() {
        let mut manager = JobManager::new();
        let id = manager.submit("success", "test", |context| {
            context.report_progress(100);
            Ok(())
        });

        wait_for_job(&mut manager, id, |job| job.status == JobStatus::Completed);
        let job = manager.job(id).unwrap();
        assert_eq!(job.progress, Some(100));
        assert!(job.started_at.is_some());
        assert!(job.finished_at.is_some());
        assert!(job.error.is_none());
    }

    #[test]
    fn failed_job_records_error() {
        let mut manager = JobManager::new();
        let id = manager.submit("failure", "test", |_| {
            Err(anyhow::anyhow!("expected failure"))
        });

        wait_for_job(&mut manager, id, |job| job.status == JobStatus::Failed);
        let job = manager.job(id).unwrap();
        assert_eq!(job.error.as_deref(), Some("expected failure"));
        assert!(job.finished_at.is_some());
    }

    #[test]
    fn progress_is_reported_and_clamped() {
        let mut manager = JobManager::new();
        let id = manager.submit("progress", "test", |context| {
            context.report_progress(45);
            context.report_progress(250);
            Ok(())
        });

        wait_for_job(&mut manager, id, |job| job.status == JobStatus::Completed);
        assert_eq!(manager.job(id).unwrap().progress, Some(100));
    }

    #[test]
    fn cooperative_cancellation_reaches_cancelled_state() {
        let mut manager = JobManager::new();
        let (release_tx, release_rx) = std_mpsc::channel();
        let release_rx = Arc::new(Mutex::new(release_rx));

        let id = manager.submit("cancel", "test", move |context| {
            while !context.is_cancelled() {
                thread::sleep(Duration::from_millis(2));
            }
            let _ = release_tx.send(());
            Ok(())
        });

        wait_for_job(&mut manager, id, |job| job.status == JobStatus::Running);
        assert!(manager.request_cancellation(id));
        wait_for_job(&mut manager, id, |job| job.status == JobStatus::Cancelled);

        let result = release_rx
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(1));
        assert!(result.is_ok());
        assert!(manager.job(id).unwrap().finished_at.is_some());
    }

    #[test]
    fn retrieve_job_by_id() {
        let mut manager = JobManager::new();
        let id = manager.submit("named job", "description", |_| Ok(()));

        let job = manager.job(id).unwrap();
        assert_eq!(job.name, "named job");
        assert_eq!(job.description, "description");
        assert_eq!(manager.jobs().len(), 1);
        assert!(manager.job(JobId(999)).is_none());
    }

    #[test]
    fn multiple_jobs_run_concurrently() {
        let mut manager = JobManager::new();
        let barrier = Arc::new(Barrier::new(3));

        let first = manager.submit("one", "test", {
            let barrier = Arc::clone(&barrier);
            move |_| {
                barrier.wait();
                Ok(())
            }
        });
        let second = manager.submit("two", "test", {
            let barrier = Arc::clone(&barrier);
            move |_| {
                barrier.wait();
                Ok(())
            }
        });

        barrier.wait();
        wait_for_job(&mut manager, first, |job| {
            job.status == JobStatus::Completed
        });
        wait_for_job(&mut manager, second, |job| {
            job.status == JobStatus::Completed
        });
    }

    #[test]
    fn terminal_state_is_not_overwritten() {
        let mut manager = JobManager::new();
        let id = manager.submit("terminal", "test", |context| {
            context.report_progress(25);
            Ok(())
        });

        wait_for_job(&mut manager, id, |job| job.status == JobStatus::Completed);

        manager.process_pending_messages();
        let completed = manager.job(id).unwrap();

        manager.process_message(WorkerMessage::Progress {
            job_id: id,
            progress: 1,
        });
        manager.process_message(WorkerMessage::Failed {
            job_id: id,
            error: "late failure".to_string(),
        });

        assert_eq!(manager.job(id).unwrap(), completed);
    }
}
