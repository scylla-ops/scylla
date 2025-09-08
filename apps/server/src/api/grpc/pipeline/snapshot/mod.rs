use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use async_trait::async_trait;
use uuid::Uuid;

pub mod controller;
pub mod models;
pub mod repo;
pub mod service;

#[async_trait]
pub trait PipelineSnapshotRepository: Send + Sync {
    async fn create_snapshot(&self, pipeline: PipelineRecord) -> anyhow::Result<Uuid>;
    async fn get_snapshot(&self, id: Uuid) -> anyhow::Result<PipelineSnapshotRecord>;
    async fn list_snapshots(
        &self,
        pipeline: PipelineRecord,
    ) -> anyhow::Result<Vec<PipelineSnapshotRecord>>;
    async fn delete_snapshot(&self, id: Uuid) -> anyhow::Result<()>;
}
