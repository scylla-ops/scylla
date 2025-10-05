use crate::api::grpc::job::models::NewJob;
use crate::api::grpc::job::repos::JobRepository;
use crate::api::grpc::orchestrator::service::ORCHESTRATOR_SERVICE;
use crate::api::grpc::pipeline::repos::PipelineRepository;
use crate::api::grpc::pipeline::snapshot::models::NewPipelineSnapshot;
use crate::api::grpc::pipeline::snapshot::repos::PipelineSnapshotRepository;
use crate::api::grpc::utils::Id;
use protocol::job::{Job, JobData, JobEntry};
use protocol::pipeline::PipelineError;
use sha2::Digest;
use thiserror::Error;

pub struct JobService<JR: JobRepository, PR: PipelineRepository, SR: PipelineSnapshotRepository> {
    _marker: std::marker::PhantomData<(JR, PR, SR)>,
}

#[derive(Debug, Error)]
pub enum JobServiceError {
    #[error("Pipeline service error: {0}")]
    PipelineService(anyhow::Error),
    #[error("Pipeline Snapshot service error: {0}")]
    PipelineSnapshotService(anyhow::Error),
    #[error("Error in job repo: {0}")]
    JobRepo(anyhow::Error),
    #[error("Pipeline error: {0}")]
    Pipeline(#[from] PipelineError),
}

pub struct JobCreationResult {
    pub job_id: Id,
    pub snapshot_id: Id,
}

impl<JR: JobRepository, PR: PipelineRepository, SR: PipelineSnapshotRepository>
    JobService<JR, PR, SR>
{
    pub async fn create_job(pipeline_id: Id) -> Result<JobCreationResult, JobServiceError> {
        let record = PR::get_pipeline(pipeline_id)
            .await
            .map_err(JobServiceError::PipelineService)?;
        let snapshots = SR::list_snapshots(&record)
            .await
            .map_err(JobServiceError::PipelineSnapshotService)?;

        let pipeline_hash = sha2::Sha256::digest(record.content.as_bytes()?);
        let snapshot_id: Id =
            match snapshots
                .into_iter()
                .find(|s| match s.content.as_bytes().ok() {
                    None => false,
                    Some(bytes) => sha2::Sha256::digest(bytes) == pipeline_hash,
                }) {
                Some(snapshot) => snapshot.id.key().to_string(),
                None => SR::create_snapshot(NewPipelineSnapshot {
                    pipeline: record.id,
                    content: record.content,
                })
                .await
                .map_err(JobServiceError::PipelineSnapshotService)?,
            };

        let snapshot = SR::get_snapshot(snapshot_id.clone())
            .await
            .map_err(JobServiceError::PipelineSnapshotService)?;

        let new_job: NewJob = NewJob {
            snapshot: snapshot.id,
            content: snapshot.content.clone().into(),
        };

        let job_id = JR::create_job(new_job)
            .await
            .map_err(JobServiceError::JobRepo)?;

        let job_record = JR::get_job(job_id.clone())
            .await
            .map_err(JobServiceError::JobRepo)?;

        let job: Job = Job::from_pipeline_and_data(snapshot.content, job_record.content);

        ORCHESTRATOR_SERVICE
            .queue_job(JobEntry {
                id: job_id.clone(),
                job,
            })
            .await;

        Ok(JobCreationResult {
            job_id,
            snapshot_id,
        })
    }

    pub async fn update_job(job_id: Id, job_data: JobData) -> Result<(), JobServiceError> {
        JR::update_job(job_id, job_data)
            .await
            .map_err(JobServiceError::JobRepo)
    }
}
