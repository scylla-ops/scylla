use crate::domain::entities::Pipeline;
use crate::domain::errors::{DomainError, DomainResult};
use crate::infrastructure::persistence::surrealdb::models::{
    PipelineInsert, PipelineRecord, PipelineUpdate,
};
use chrono::DateTime;

impl TryFrom<PipelineRecord> for Pipeline {
    type Error = DomainError;

    /// Convert database record to domain entity
    fn try_from(record: PipelineRecord) -> DomainResult<Self> {
        let id = record.id.key().to_string();
        let name = record.name;
        let nodes = record.nodes;

        Ok(Pipeline::new(
            id,
            name,
            nodes,
            DateTime::from(record.created_at),
            DateTime::from(record.updated_at),
        ))
    }
}

impl From<&Pipeline> for PipelineInsert {
    /// Convert domain entity to insert record
    fn from(pipeline: &Pipeline) -> Self {
        PipelineInsert {
            name: pipeline.name().to_string(),
            nodes: pipeline.nodes().to_vec(),
        }
    }
}

impl From<&Pipeline> for PipelineUpdate {
    /// Convert domain entity to update record
    fn from(pipeline: &Pipeline) -> Self {
        PipelineUpdate {
            name: pipeline.name().to_string(),
            nodes: pipeline.nodes().to_vec(),
        }
    }
}
