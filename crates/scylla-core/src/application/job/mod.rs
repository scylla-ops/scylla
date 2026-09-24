pub mod commands;
pub mod log_repository;
pub mod log_stream_port;
pub mod log_stream_use_case;
pub mod log_use_case;
pub mod queries;
pub mod reaper;
pub mod repository;

pub use scylla_domain::JobEvent;

pub use commands::{DeleteJob, RecordJobStatus};
pub use log_repository::JobLogRepository;
pub use log_stream_port::{JobLogLiveStream, JobLogStreamPort};
pub use log_stream_use_case::JobLogStreamUseCase;
pub use log_use_case::JobLogUseCases;
pub use queries::{GetJob, ListJobs, ListOrganizationJobs, ListPipelineJobs, ListProjectJobs};
pub use reaper::JobReaper;
pub use repository::JobRepository;

use derive_more::Constructor;
use std::sync::Arc;

/// The job's stage runners, one block per action in `commands.rs` and `queries.rs`. The agent
/// stream sends `RecordJobStatus` through the engine; the reaper and the dispatch write through
/// the port directly.
#[derive(Constructor)]
pub struct JobUseCases<J: JobRepository> {
    pub(super) job_repo: Arc<J>,
}

#[cfg(test)]
mod tests;
