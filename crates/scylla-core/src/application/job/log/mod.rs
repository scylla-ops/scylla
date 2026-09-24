pub mod commands;
pub mod queries;
pub mod repository;
pub mod stream_port;

pub use commands::AppendJobLog;
pub use queries::{ListJobLogs, TailJobLogs};
pub use repository::JobLogRepository;
pub use stream_port::{JobLogLiveStream, JobLogStreamPort};

use derive_more::Constructor;
use std::sync::Arc;

/// The job log's stage runners, one block per action in `commands.rs` and `queries.rs`. The
/// agent stream sends `AppendJobLog` through the engine; the live fan-out stays in the stream.
#[derive(Constructor)]
pub struct JobLogUseCases<L: JobLogRepository, S: JobLogStreamPort> {
    pub(super) log_repo: Arc<L>,
    pub(super) stream_port: Arc<S>,
}

#[cfg(test)]
mod tests;
