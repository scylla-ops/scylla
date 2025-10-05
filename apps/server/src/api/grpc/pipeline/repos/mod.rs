use crate::api::grpc::pipeline::models::{NewPipeline, PipelinePatch, PipelineRecord};
use crate::api::grpc::utils::Id;
use async_trait::async_trait;

#[cfg(feature = "surreal")]
pub mod surreal;

const TABLE: &str = "pipelines";

#[async_trait]
pub trait PipelineRepository: Send + Sync + 'static {
    async fn create_pipeline(new_pipeline: NewPipeline) -> anyhow::Result<Id>;
    async fn get_pipeline(id: Id) -> anyhow::Result<PipelineRecord>;
    async fn list_pipelines() -> anyhow::Result<Vec<PipelineRecord>>;
    async fn delete_pipeline(id: Id) -> anyhow::Result<()>;
    async fn update_pipeline(id: Id, patch: PipelinePatch) -> anyhow::Result<()>;
}
