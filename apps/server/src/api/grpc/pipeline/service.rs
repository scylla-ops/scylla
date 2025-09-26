use crate::api::grpc::pipeline::PipelineRepository;
use crate::api::grpc::pipeline::models::PipelineRecord as DbPipelineRecord;
use crate::api::grpc::pipeline::repo::PipelineRepositoryDiesel;
use crate::database::get_existing_db;
use derive_more::Constructor;
use protocol::pipeline::Pipeline;
use protocol::toml;
use std::sync::{Arc, LazyLock};
use thiserror::Error;
use uuid::Uuid;

#[derive(Constructor, Clone)]
pub struct PipelineService {
    repo: Arc<dyn PipelineRepository>,
}

#[derive(Debug, Error)]
pub enum PipelineServiceError {
    #[error("Invalid pipeline TOML: {0}")]
    InvalidToml(String),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

pub static PIPELINE_SERVICE: LazyLock<Arc<PipelineService>> = LazyLock::new(|| {
    let diesel_db = get_existing_db();

    Arc::new(PipelineService::new(Arc::new(
        PipelineRepositoryDiesel::new(diesel_db.clone()),
    )))
});

impl PipelineService {
    pub async fn create_pipeline(&self, pipeline_toml: &str) -> Result<Uuid, PipelineServiceError> {
        let parsed_pipeline: Pipeline = toml::from_str(pipeline_toml)
            .map_err(|e| PipelineServiceError::InvalidToml(e.to_string()))?;
        let id = self
            .repo
            .create_pipeline(parsed_pipeline)
            .await
            .map_err(PipelineServiceError::Repo)?;
        Ok(id)
    }

    pub async fn get_pipeline(&self, id: Uuid) -> Result<DbPipelineRecord, PipelineServiceError> {
        let record = self
            .repo
            .get_pipeline(id)
            .await
            .map_err(PipelineServiceError::Repo)?;
        // Validate content parses as Pipeline
        let _parsed: Pipeline = toml::from_str(&record.content)
            .map_err(|e| PipelineServiceError::InvalidToml(e.to_string()))?;
        Ok(record)
    }

    pub async fn delete_pipeline(&self, id: Uuid) -> Result<(), PipelineServiceError> {
        self.repo
            .delete_pipeline(id)
            .await
            .map_err(PipelineServiceError::Repo)
    }

    pub async fn update_pipeline(
        &self,
        id: Uuid,
        pipeline_toml: &str,
    ) -> Result<(), PipelineServiceError> {
        let parsed_pipeline: Pipeline = toml::from_str(pipeline_toml)
            .map_err(|e| PipelineServiceError::InvalidToml(e.to_string()))?;
        self.repo
            .update_pipeline(id, parsed_pipeline)
            .await
            .map_err(PipelineServiceError::Repo)
    }
}
