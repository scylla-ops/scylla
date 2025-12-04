use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PipelineContent, PipelineId};
use chrono::{DateTime, Utc};
use derive_more::Constructor;

/// Pipeline domain entity
#[derive(Debug, Clone, Constructor)]
pub struct Pipeline {
    id: PipelineId,
    content: PipelineContent,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Pipeline {
    /// Create a new pipeline
    pub fn create(content: PipelineContent) -> DomainResult<Self> {
        let now = Utc::now();
        Ok(Self {
            id: PipelineId::generate(),
            content,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update the pipeline content
    pub fn update_content(&mut self, content: PipelineContent) -> DomainResult<()> {
        self.content = content;
        self.updated_at = Utc::now();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &PipelineId {
        &self.id
    }

    pub fn content(&self) -> &PipelineContent {
        &self.content
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
