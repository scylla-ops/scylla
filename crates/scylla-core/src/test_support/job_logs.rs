//! `JobLog` test fixtures.

use bon::bon;
use chrono::{DateTime, Utc};

use crate::domain::clock;
use crate::domain::entities::{JobId, JobLog, JobLogId};
use crate::domain::value_objects::job::LogStream;
use crate::domain::value_objects::pipeline::NodeId;

pub struct JobLogBuilder;

#[bon]
#[allow(clippy::new_ret_no_self, clippy::must_use_candidate)]
impl JobLogBuilder {
    #[builder(start_fn = new, finish_fn = build)]
    pub fn assemble(
        #[builder(start_fn)] job_id: &JobId,
        #[builder(start_fn, into)] node_id: String,
        #[builder(start_fn, into)] line: String,
        id: Option<JobLogId>,
        #[builder(default = LogStream::Stdout)] stream: LogStream,
        timestamp: Option<DateTime<Utc>>,
        created_at: Option<DateTime<Utc>>,
    ) -> JobLog {
        let ts = timestamp.unwrap_or_else(clock::now);
        JobLog::from_persistence(
            id.unwrap_or_else(JobLogId::generate),
            job_id.clone(),
            NodeId::new(node_id).expect("test node id invalid"),
            stream,
            line,
            ts,
            created_at.unwrap_or(ts),
        )
    }
}

#[must_use]
pub fn job_log(job_id: &JobId, node_id: &str, line: &str) -> JobLog {
    JobLogBuilder::new(job_id, node_id, line).build()
}

#[cfg(feature = "postgres")]
pub async fn seed_job_log(
    pool: &sqlx::PgPool,
    job_id: &JobId,
    node_id: &str,
    line: &str,
) -> JobLog {
    use crate::application::JobLogRepository;
    use crate::infrastructure::persistence::postgres::PgJobLogRepository;
    let log = job_log(job_id, node_id, line);
    PgJobLogRepository::new(pool.clone())
        .create(&log)
        .await
        .expect("seed job log failed");
    log
}
