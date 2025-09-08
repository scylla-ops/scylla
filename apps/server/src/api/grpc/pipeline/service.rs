use crate::api::grpc::pipeline::PipelineRepository;
use crate::api::grpc::pipeline::models::PipelineRecord as DbPipelineRecord;
use derive_more::Constructor;
use protocol::pipeline::Pipeline;
use protocol::toml;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Constructor, Clone)]
pub struct PipelineService {
    repo: Arc<dyn PipelineRepository>,
}

#[derive(Debug, Error)]
pub enum PipelineDomainError {
    #[error("Invalid pipeline TOML: {0}")]
    InvalidToml(String),
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),
    #[error("Pipeline not found")]
    NotFound,
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

impl PipelineService {
    pub async fn create_pipeline(&self, pipeline_toml: &str) -> Result<Uuid, PipelineDomainError> {
        let parsed_pipeline: Pipeline = toml::from_str(pipeline_toml)
            .map_err(|e| PipelineDomainError::InvalidToml(e.to_string()))?;
        let id = self
            .repo
            .create_pipeline(parsed_pipeline)
            .await
            .map_err(PipelineDomainError::Repo)?;
        Ok(id)
    }

    pub async fn get_pipeline(&self, id: Uuid) -> Result<DbPipelineRecord, PipelineDomainError> {
        let record = self
            .repo
            .get_pipeline(id)
            .await
            .map_err(PipelineDomainError::Repo)?;
        // Validate content parses as Pipeline
        let _parsed: Pipeline = toml::from_str(&record.content)
            .map_err(|e| PipelineDomainError::InvalidToml(e.to_string()))?;
        Ok(record)
    }

    pub async fn delete_pipeline(&self, id: Uuid) -> Result<(), PipelineDomainError> {
        self.repo
            .delete_pipeline(id)
            .await
            .map_err(PipelineDomainError::Repo)
    }

    pub async fn update_pipeline(
        &self,
        id: Uuid,
        pipeline_toml: &str,
    ) -> Result<(), PipelineDomainError> {
        let parsed_pipeline: Pipeline = toml::from_str(pipeline_toml)
            .map_err(|e| PipelineDomainError::InvalidToml(e.to_string()))?;
        self.repo
            .update_pipeline(id, parsed_pipeline)
            .await
            .map_err(PipelineDomainError::Repo)
    }
}
