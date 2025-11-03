use crate::domain::entities::Job;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{JobContent, JobId, JobStatus, PipelineId};
use crate::infrastructure::persistence::JobUpdate;
use crate::infrastructure::persistence::surrealdb::mappers::{FromRecordId, ToRecordId};
use crate::infrastructure::persistence::surrealdb::models::{JobInsert, JobRecord};
use chrono::DateTime;
use std::convert::{From, TryFrom};

impl TryFrom<JobRecord> for Job {
    type Error = DomainError;

    /// Convert database record to domain entity
    fn try_from(record: JobRecord) -> DomainResult<Self> {
        let id = JobId::from_record_id(record.id);
        let pipeline_id = PipelineId::from_record_id(record.pipeline_id);
        let status = JobStatus::new(&record.status)?;
        let content = JobContent::new(record.content)?;

        Ok(Job::new(
            id,
            pipeline_id,
            status,
            content,
            DateTime::from(record.created_at),
            DateTime::from(record.updated_at),
        ))
    }
}

impl From<&Job> for JobInsert {
    /// Convert domain entity to insert record
    fn from(job: &Job) -> Self {
        JobInsert {
            pipeline_id: job.pipeline_id().to_record_id(),
            status: job.status().as_str().to_string(),
            content: job.content().as_str().to_string(),
        }
    }
}

impl From<&Job> for JobUpdate {
    /// Convert domain entity to update record
    fn from(job: &Job) -> Self {
        JobUpdate {
            status: job.status().as_str().to_string(),
        }
    }
}
