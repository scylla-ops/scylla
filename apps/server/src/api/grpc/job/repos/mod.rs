use crate::api::grpc::job::models::{JobRecord, NewJob};
use crate::api::grpc::utils::Id;
use async_trait::async_trait;
use protocol::job::JobData;

#[cfg(feature = "surreal")]
pub mod surreal;

#[async_trait]
pub trait JobRepository: Send + Sync + 'static {
    const TABLE: &'static str = "jobs";

    async fn create_job(job: NewJob) -> anyhow::Result<Id>;
    async fn get_job(job_id: Id) -> anyhow::Result<JobRecord>;
    async fn update_job(job_id: Id, job_data: JobData) -> anyhow::Result<()>;

    /*
    async fn update_stage(stage_id: Id, updated_stage: StageStatusUpdate) -> anyhow::Result<()>;

    async fn update_step(step_id: Id, updated_step: StepStatusUpdate) -> anyhow::Result<()>;*/
}
