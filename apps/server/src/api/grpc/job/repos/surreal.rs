use crate::api::grpc::job::models::{JobRecord, JobUpdate, NewJob};
use crate::api::grpc::job::repos::JobRepository;
use crate::api::grpc::utils::Id;
use crate::database::db;
use anyhow::Context;
use async_trait::async_trait;
use protocol::job::JobData;
use surrealdb::RecordId;

pub struct JobRepositorySurreal;

#[async_trait]
impl JobRepository for JobRepositorySurreal {
    async fn create_job(job: NewJob) -> anyhow::Result<Id> {
        #[derive(serde::Deserialize)]
        struct JobResponse {
            id: RecordId,
        }

        let newjob: Option<JobResponse> = db()
            .create(Self::TABLE)
            .content(job)
            .await
            .context("Failed to create job")?;

        Ok(newjob.context("Failed to get job id")?.id.key().to_string())
    }

    async fn get_job(job_id: Id) -> anyhow::Result<JobRecord> {
        let job: Option<JobRecord> = db()
            .select((Self::TABLE, job_id))
            .await
            .context("Failed to get job")?;

        job.context("Job not found")
    }

    async fn update_job(job_id: Id, job_data: JobData) -> anyhow::Result<()> {
        let _: Option<JobRecord> = db()
            .update((Self::TABLE, job_id))
            .content(JobUpdate { content: job_data })
            .await
            .context("Failed to update job")?;
        Ok(())
    }
}
