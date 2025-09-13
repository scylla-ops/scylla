pub mod controller;
pub mod models;
pub mod repo;
pub mod service;

use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create_job(
        &self,
        snapshot_id: Uuid,
        pipeline: protocol::pipeline::Pipeline,
    ) -> anyhow::Result<Uuid>;
}
