use crate::domain::entities::{Job, JobNodeExecution};
use crate::domain::value_objects::{JobStatus, PaginationMetadata, PaginationParams};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DTO pour créer un job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequestDto {
    pub pipeline_id: String,
}

/// DTO pour récupérer un job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetJobRequestDto {
    pub job_id: String,
}

/// DTO pour mettre à jour un job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJobRequestDto {
    pub job_id: String,
    pub status: Option<JobStatus>,
}

/// DTO pour l'exécution d'un nœud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobNodeExecutionDto {
    pub node_id: String,
    pub state: JobStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl From<&JobNodeExecution> for JobNodeExecutionDto {
    fn from(execution: &JobNodeExecution) -> Self {
        Self {
            node_id: execution.node_id().as_str().to_string(),
            state: execution.state().clone(),
            started_at: execution.started_at(),
            finished_at: execution.finished_at(),
        }
    }
}

/// DTO de réponse pour un job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponseDto {
    pub id: String,
    pub pipeline_id: String,
    pub status: JobStatus,
    pub executions: HashMap<String, JobNodeExecutionDto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Job> for JobResponseDto {
    fn from(job: Job) -> Self {
        Self {
            id: job.id().as_str().to_string(),
            pipeline_id: job.pipeline_id().to_string(),
            status: *job.status(),
            executions: job
                .executions()
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), JobNodeExecutionDto::from(v)))
                .collect(),
            created_at: job.created_at(),
            updated_at: job.updated_at(),
        }
    }
}

impl From<&Job> for JobResponseDto {
    fn from(job: &Job) -> Self {
        Self {
            id: job.id().as_str().to_string(),
            pipeline_id: job.pipeline_id().to_string(),
            status: *job.status(),
            executions: job
                .executions()
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), JobNodeExecutionDto::from(v)))
                .collect(),
            created_at: job.created_at(),
            updated_at: job.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteJobRequestDto {
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteJobResponseDto {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListJobsRequestDto {
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListJobsResponseDto {
    pub jobs: Vec<JobResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListJobsByStatusRequestDto {
    pub status: JobStatus,
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListJobsByStatusResponseDto {
    pub jobs: Vec<JobResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListJobsByPipelineRequestDto {
    pub pipeline_id: String,
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListJobsByPipelineResponseDto {
    pub jobs: Vec<JobResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}
