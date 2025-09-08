use crate::api::grpc::pipeline::models::PipelineRecord;
use async_trait::async_trait;
use derive_more::Constructor;
use protocol::pipeline::Pipeline;
use std::sync::Arc;
use uuid::Uuid;

pub mod controller;
pub mod models;
pub mod repo;
pub mod service;
pub mod snapshot;
pub mod worker;

#[async_trait]
pub trait PipelineRepository: Send + Sync {
    async fn create_pipeline(&self, pipeline: Pipeline) -> anyhow::Result<Uuid>;
    async fn get_pipeline(&self, id: Uuid) -> anyhow::Result<PipelineRecord>;
    async fn list_pipelines(&self) -> anyhow::Result<Vec<Pipeline>>;
    async fn delete_pipeline(&self, id: Uuid) -> anyhow::Result<()>;
    async fn update_pipeline(&self, id: Uuid, updated_pipeline: Pipeline) -> anyhow::Result<()>;
}
