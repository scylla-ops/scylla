use crate::domain::entities::Pipeline;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PipelineContent, PipelineId};
use crate::infrastructure::persistence::PipelineUpdate;
use crate::infrastructure::persistence::surrealdb::models::{PipelineInsert, PipelineRecord};
use chrono::DateTime;

/// Mapper between Pipeline domain entity and database records
pub struct PipelineMapper;

impl PipelineMapper {
    /// Convert database record to domain entity
    pub fn to_domain(record: PipelineRecord) -> DomainResult<Pipeline> {
        let id = PipelineId::new(record.id.key().to_string());
        let content = PipelineContent::new(record.content)?;

        Pipeline::new(
            id,
            content,
            DateTime::from(record.created_at),
            DateTime::from(record.updated_at),
        )
    }

    /// Convert domain entity to insert record
    pub fn to_insert(pipeline: &Pipeline) -> PipelineInsert {
        PipelineInsert {
            content: pipeline.content().as_str().to_string(),
        }
    }

    /// Convert domain entity to update record
    pub fn to_update(pipeline: &Pipeline) -> PipelineUpdate {
        PipelineUpdate {
            content: pipeline.content().as_str().to_string(),
        }
    }
}
