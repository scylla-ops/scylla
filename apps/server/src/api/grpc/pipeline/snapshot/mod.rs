use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use crate::api::grpc::pipeline::worker::PipelineMessage;
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

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

#[derive(Constructor)]
pub struct PipelineSnapshotService {
    repo: Arc<dyn PipelineSnapshotRepository>,
    tx_pipeline: mpsc::Sender<PipelineMessage>,
}
