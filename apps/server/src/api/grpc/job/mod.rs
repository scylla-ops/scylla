pub mod controller;
pub mod models;
pub mod repo;
pub mod service;

use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create_job(&self, pipeline_id: Uuid) -> anyhow::Result<Uuid>;
}

#[async_trait]
pub trait StageRepository: Send + Sync {
    async fn create_stages(&self, job_id: Uuid) -> anyhow::Result<()>;
}

#[async_trait]
pub trait StepRepository: Send + Sync {
    async fn create_steps(&self, stage_id: Uuid) -> anyhow::Result<()>;
}
