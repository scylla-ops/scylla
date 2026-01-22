use crate::domain::entities::Pipeline;
use crate::domain::value_objects::{PaginationMetadata, PaginationParams, PipelineId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct CreatePipelineRequestDto {
    pub content: Pipeline,
}

#[derive(Debug, Clone)]
pub struct GetPipelineRequestDto {
    pub pipeline_id: PipelineId,
}

#[derive(Debug, Clone)]
pub struct UpdatePipelineRequestDto {
    pub pipeline_id: PipelineId,
    pub content: Pipeline,
}

#[derive(Debug, Clone)]
pub struct PipelineResponseDto {
    pub id: PipelineId,
    pub content: Pipeline,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Pipeline> for PipelineResponseDto {
    fn from(pipeline: Pipeline) -> Self {
        Self {
            id: pipeline.id().to_owned(),
            content: pipeline.to_owned(),
            created_at: pipeline.created_at(),
            updated_at: pipeline.updated_at(),
        }
    }
}

impl From<&Pipeline> for PipelineResponseDto {
    fn from(pipeline: &Pipeline) -> Self {
        Self {
            id: pipeline.id().to_owned(),
            content: pipeline.content().to_owned(),
            created_at: pipeline.created_at(),
            updated_at: pipeline.updated_at(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeletePipelineRequestDto {
    pub pipeline_id: PipelineId,
}

#[derive(Debug, Clone)]
pub struct DeletePipelineResponseDto {}

#[derive(Debug, Clone)]
pub struct ListPipelinesRequestDto {
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ListPipelinesResponseDto {
    pub pipelines: Vec<PipelineResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}
