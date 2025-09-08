pub mod models;
pub mod repo;
pub mod service;

use async_trait::async_trait;
use derive_more::Constructor;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Constructor, Debug)]
pub struct JobService {
    job_repo: Arc<dyn JobRepository>,
    stage_repo: Arc<dyn StageRepository>,
    step_repo: Arc<dyn StepRepository>,
}

#[async_trait]
pub trait JobRepository: Send + Sync + Debug {
    async fn create_job(&self, pipeline_id: Uuid) -> anyhow::Result<Uuid>;
}

#[async_trait]
pub trait StageRepository: Send + Sync + Debug {
    async fn create_stages(&self, job_id: Uuid) -> anyhow::Result<()>;
}

#[async_trait]
pub trait StepRepository: Send + Sync + Debug {
    async fn create_steps(&self, stage_id: Uuid) -> anyhow::Result<()>;
}
