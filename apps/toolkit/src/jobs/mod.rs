mod manager;
mod model;
mod view;

pub use manager::{JobContext, JobManager};
pub use model::{Job, JobId, JobStatus};
pub use view::render;
