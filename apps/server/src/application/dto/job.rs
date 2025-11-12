use crate::domain::entities::Job;
use crate::domain::value_objects::{JobContent, JobStatus};
use crate::domain::value_objects::{JobId, PaginationMetadata, PaginationParams, PipelineId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct CreateJobRequestDto {
    pub pipeline_id: PipelineId,
}

#[derive(Debug, Clone)]
pub struct GetJobRequestDto {
    pub job_id: JobId,
}

#[derive(Debug, Clone)]
pub struct UpdateJobRequestDto {
    pub job_id: JobId,
    pub status: Option<JobStatus>,
}

#[derive(Debug, Clone)]
pub struct JobResponseDto {
    pub id: JobId,
    pub pipeline_id: PipelineId,
    pub status: JobStatus,
    pub content: JobContent,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Job> for JobResponseDto {
    fn from(job: Job) -> Self {
        Self {
            id: job.id().to_owned(),
            pipeline_id: job.pipeline_id().to_owned(),
            status: job.status().to_owned(),
            content: job.content().to_owned(),
            created_at: job.created_at(),
            updated_at: job.updated_at(),
        }
    }
}

impl From<&Job> for JobResponseDto {
    fn from(job: &Job) -> Self {
        Self {
            id: job.id().to_owned(),
            pipeline_id: job.pipeline_id().to_owned(),
            status: job.status().to_owned(),
            content: job.content().to_owned(),
            created_at: job.created_at(),
            updated_at: job.updated_at(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteJobRequestDto {
    pub job_id: JobId,
}

#[derive(Debug, Clone)]
pub struct DeleteJobResponseDto {}

#[derive(Debug, Clone)]
pub struct ListJobsRequestDto {
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ListJobsResponseDto {
    pub jobs: Vec<JobResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone)]
pub struct ListJobsByStatusRequestDto {
    pub status: JobStatus,
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ListJobsByStatusResponseDto {
    pub jobs: Vec<JobResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone)]
pub struct ListJobsByPipelineRequestDto {
    pub pipeline_id: PipelineId,
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ListJobsByPipelineResponseDto {
    pub jobs: Vec<JobResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}
