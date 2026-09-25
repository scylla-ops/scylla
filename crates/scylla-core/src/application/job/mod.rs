pub mod commands;
pub mod log;
pub mod queries;
pub mod reaper;
pub mod repository;

pub use scylla_domain::JobEvent;

pub use commands::{DeleteJob, ReapOrphanedJobs, RecordJobStatus};
pub use log::{
    AppendJobLog, JobLogLiveStream, JobLogRepository, JobLogStreamPort, JobLogUseCases,
    ListJobLogs, TailJobLogs,
};
pub use queries::{GetJob, ListJobs, ListOrganizationJobs, ListPipelineJobs, ListProjectJobs};
pub use reaper::JobReaper;
pub use repository::JobRepository;

use derive_more::Constructor;
use std::sync::Arc;

/// The job's stage runners, one block per action in `commands.rs` and `queries.rs`. The agent
/// stream sends `RecordJobStatus`, and `JobReaper` sends `ReapOrphanedJobs`.
#[derive(Constructor)]
pub struct JobUseCases {
    pub(super) job_repo: Arc<dyn JobRepository>,
}

#[cfg(test)]
mod tests;
