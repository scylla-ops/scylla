use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{JobContent, JobId, JobStatus, PipelineContent, PipelineId};
use chrono::{DateTime, Utc};

/// Job domain entity
#[derive(Debug, Clone)]
pub struct Job {
    id: JobId,
    pipeline_id: PipelineId,
    status: JobStatus,
    content: JobContent,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Job {
    /// Create a new job (for reconstruction from database)
    pub fn new(
        id: JobId,
        pipeline_id: PipelineId,
        status: JobStatus,
        content: JobContent,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            pipeline_id,
            status,
            content,
            created_at,
            updated_at,
        }
    }

    /// Create a new job
    pub fn create(pipeline_id: PipelineId, content: PipelineContent) -> DomainResult<Self> {
        let now = Utc::now();
        Ok(Self {
            id: JobId::generate(),
            pipeline_id,
            status: JobStatus::pending(),
            content: JobContent::new(content.into_string())?,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update job status with validation
    pub fn update_status(&mut self, new_status: JobStatus) -> DomainResult<()> {
        self.status.validate_transition_to(&new_status)?;
        self.status = new_status;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Start the job (transition from Pending to Running)
    pub fn start(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::running())
    }

    /// Mark job as completed
    pub fn complete(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::completed())
    }

    /// Mark job as failed
    pub fn fail(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::failed())
    }

    /// Cancel the job
    pub fn cancel(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::cancelled())
    }

    /// Update job content
    pub fn update_content(&mut self, content: JobContent) -> DomainResult<()> {
        if self.status.is_terminal() {
            return Err(DomainError::business_rule(
                "Cannot update content of a terminal job",
            ));
        }

        self.content = content;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if job can be cancelled
    pub fn can_cancel(&self) -> bool {
        self.status == JobStatus::pending() || self.status == JobStatus::running()
    }

    // Getters
    pub fn id(&self) -> &JobId {
        &self.id
    }

    pub fn pipeline_id(&self) -> &PipelineId {
        &self.pipeline_id
    }

    pub fn status(&self) -> &JobStatus {
        &self.status
    }

    pub fn content(&self) -> &JobContent {
        &self.content
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
}
