use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::snapshot::models::{NewPipelineSnapshot, PipelineSnapshotRecord};
use crate::api::grpc::utils::Id;
use async_trait::async_trait;

#[cfg(feature = "surreal")]
pub mod surreal;

const TABLE: &str = "pipeline_snapshots";

#[async_trait]
pub trait PipelineSnapshotRepository: Send + Sync + 'static {
    async fn create_snapshot(pipeline: NewPipelineSnapshot) -> anyhow::Result<Id>;
    async fn get_snapshot(id: Id) -> anyhow::Result<PipelineSnapshotRecord>;
    async fn list_snapshots(
        pipeline: &PipelineRecord,
    ) -> anyhow::Result<Vec<PipelineSnapshotRecord>>;
    async fn delete_snapshot(id: Id) -> anyhow::Result<()>;
}
