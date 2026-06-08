//! `Job` test fixtures.

use bon::bon;
use chrono::{DateTime, Utc};

use crate::domain::clock;
use crate::domain::entities::{Job, JobId, JobNode, Pipeline};
use crate::domain::value_objects::job::{JobStatus, NodeState};

pub struct JobBuilder;

#[bon]
#[allow(clippy::new_ret_no_self, clippy::must_use_candidate)]
impl JobBuilder {
    /// Build a fresh `Pending` job mirroring `pipeline.nodes()`.
    #[builder(start_fn = new, finish_fn = build)]
    pub fn assemble(
        #[builder(start_fn)] pipeline: &Pipeline,
        id: Option<JobId>,
        #[builder(default = JobStatus::Pending)] status: JobStatus,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
        started_at: Option<DateTime<Utc>>,
        finished_at: Option<DateTime<Utc>>,
        /// Convenience: marks the job as `Running` with `started_at = now`
        /// (only applied if `status` is left at its default).
        #[builder(default = false)]
        running: bool,
        /// Convenience: terminal status with synthesized started/finished timestamps.
        terminated: Option<JobStatus>,
    ) -> Job {
        let pipeline_id = pipeline.id().clone();
        let node_executions: Vec<JobNode> = pipeline
            .nodes()
            .iter()
            .map(|n| JobNode::from_persistence(n.id().clone(), NodeState::Pending, None, None))
            .collect();

        let (status, started_at, finished_at) = if let Some(terminal) = terminated {
            let now = clock::now();
            (
                terminal,
                Some(now - chrono::Duration::seconds(1)),
                Some(now),
            )
        } else if running {
            (JobStatus::Running, Some(clock::now()), finished_at)
        } else {
            (status, started_at, finished_at)
        };

        let now = created_at.unwrap_or_else(clock::now);
        Job::from_persistence(
            id.unwrap_or_else(JobId::generate),
            pipeline_id,
            status,
            node_executions,
            None,
            now,
            updated_at.unwrap_or(now),
            started_at,
            finished_at,
        )
    }
}

#[must_use]
pub fn job(pipeline: &Pipeline) -> Job {
    JobBuilder::new(pipeline).build()
}

#[cfg(feature = "postgres")]
pub async fn seed_job(pool: &sqlx::PgPool, pipeline: &Pipeline) -> Job {
    use crate::application::JobRepository;
    use crate::infrastructure::persistence::postgres::PgJobRepository;
    let job = job(pipeline);
    PgJobRepository::new(pool.clone())
        .create(&job)
        .await
        .expect("seed job failed");
    job
}
