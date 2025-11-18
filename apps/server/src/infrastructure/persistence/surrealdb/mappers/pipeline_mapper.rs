use crate::domain::entities::Pipeline;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{PipelineContent, PipelineId};
use crate::infrastructure::persistence::PipelineUpdate;
use crate::infrastructure::persistence::surrealdb::models::{PipelineInsert, PipelineRecord};
use chrono::DateTime;

impl TryFrom<PipelineRecord> for Pipeline {
    type Error = DomainError;

    /// Convert database record to domain entity
    fn try_from(record: PipelineRecord) -> DomainResult<Self> {
        let id = PipelineId::new(record.id.key().to_string());
        let content = PipelineContent::new(record.content)?;

        Ok(Pipeline::new(
            id,
            content,
            DateTime::from(record.created_at),
            DateTime::from(record.updated_at),
        ))
    }
}

impl From<&Pipeline> for PipelineInsert {
    /// Convert domain entity to insert record
    fn from(pipeline: &Pipeline) -> Self {
        PipelineInsert {
            content: pipeline.content().as_str().to_string(),
        }
    }
}

impl From<&Pipeline> for PipelineUpdate {
    /// Convert domain entity to update record
    fn from(pipeline: &Pipeline) -> Self {
        PipelineUpdate {
            content: pipeline.content().as_str().to_string(),
        }
    }
}
