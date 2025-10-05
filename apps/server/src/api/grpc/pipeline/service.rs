use crate::api::grpc::pipeline::models::{NewPipeline, PipelinePatch, PipelineRecord};
use crate::api::grpc::pipeline::repos::PipelineRepository;
use derive_more::Constructor;
use protocol::pipeline::Pipeline;
use protocol::toml;
use thiserror::Error;

#[derive(Constructor, Clone)]
pub struct PipelineService<R: PipelineRepository> {
    _marker: std::marker::PhantomData<R>,
}

#[derive(Debug, Error)]
pub enum PipelineServiceError {
    #[error("Invalid pipeline TOML: {0}")]
    InvalidToml(String),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

impl<R: PipelineRepository> PipelineService<R> {
    pub async fn create_pipeline(pipeline_toml: &str) -> Result<String, PipelineServiceError> {
        let parsed_pipeline: Pipeline = toml::from_str(pipeline_toml)
            .map_err(|e| PipelineServiceError::InvalidToml(e.to_string()))?;
        let id = R::create_pipeline(NewPipeline {
            content: parsed_pipeline,
        })
        .await?;
        Ok(id)
    }

    pub async fn list_pipelines() -> Result<Vec<PipelineRecord>, PipelineServiceError> {
        let records = R::list_pipelines().await?;
        Ok(records)
    }

    pub async fn get_pipeline(id: String) -> Result<PipelineRecord, PipelineServiceError> {
        let record = R::get_pipeline(id).await?;
        Ok(record)
    }

    pub async fn delete_pipeline(id: String) -> Result<(), PipelineServiceError> {
        R::delete_pipeline(id)
            .await
            .map_err(PipelineServiceError::Repo)
    }

    pub async fn update_pipeline(
        id: String,
        pipeline_toml: &str,
    ) -> Result<(), PipelineServiceError> {
        let parsed_pipeline: Pipeline = toml::from_str(pipeline_toml)
            .map_err(|e| PipelineServiceError::InvalidToml(e.to_string()))?;
        R::update_pipeline(
            id,
            PipelinePatch {
                content: Some(parsed_pipeline),
            },
        )
        .await
        .map_err(PipelineServiceError::Repo)
    }
}
