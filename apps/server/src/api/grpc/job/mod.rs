pub mod controller;
pub mod models;
pub mod repo;
pub mod service;
pub mod worker;

use crate::api::grpc::job::models::{JobStatusUpdate, StageStatusUpdate, StepStatusUpdate};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create_job(
        &self,
        snapshot_id: Uuid,
        pipeline: &protocol::job::Job,
    ) -> anyhow::Result<Uuid>;

    async fn update_job(&self, job_id: Uuid, updated_job: JobStatusUpdate) -> anyhow::Result<()>;

    async fn update_stage(
        &self,
        stage_id: Uuid,
        updated_stage: StageStatusUpdate,
    ) -> anyhow::Result<()>;

    async fn update_step(
        &self,
        step_id: Uuid,
        updated_step: StepStatusUpdate,
    ) -> anyhow::Result<()>;
}
