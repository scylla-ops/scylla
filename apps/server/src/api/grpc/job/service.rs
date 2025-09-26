use crate::api::grpc::job::JobRepository;
use crate::api::grpc::job::models::{
    ExecutionStatus, JobStatusUpdate, StageStatusUpdate, StepStatusUpdate,
};
use crate::api::grpc::job::repo::JobRepositoryDiesel;
use crate::api::grpc::job::service::JobServiceError as E;
use crate::api::grpc::orchestrator::service::ORCHESTRATOR_SERVICE;
use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::service::PIPELINE_SERVICE;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use crate::api::grpc::pipeline::snapshot::service::PIPELINE_SNAPSHOT_SERVICE;
use crate::database::get_existing_db;
use derive_more::Constructor;
use protocol::job::Job;
use protocol::pipeline::Pipeline;
use protocol::toml;
use sha2::Digest;
use std::sync::{Arc, LazyLock};
use thiserror::Error;
use uuid::Uuid;

#[derive(Constructor)]
pub struct JobService {
    job_repo: Arc<dyn JobRepository>,
}

pub static JOB_SERVICE: LazyLock<Arc<JobService>> = LazyLock::new(|| {
    let diesel_db = get_existing_db();

    Arc::new(JobService::new(Arc::new(JobRepositoryDiesel::new(
        diesel_db.clone(),
    ))))
});

#[derive(Debug, Error)]
pub enum JobServiceError {
    #[error("Failed to parse pipeline: {0}")]
    ParsePipeline(#[from] toml::de::Error),
    #[error("Pipeline service error: {0}")]
    PipelineService(anyhow::Error),
    #[error("Pipeline Snapshot service error: {0}")]
    PipelineSnapshotService(anyhow::Error),
    #[error("Error in job repo: {0}")]
    JobRepo(anyhow::Error),
}

pub struct JobCreationResult {
    pub job_id: Uuid,
    pub snapshot_id: Uuid,
}

impl JobService {
    async fn get_pipeline(&self, pipeline_id: Uuid) -> Result<PipelineRecord, JobServiceError> {
        PIPELINE_SERVICE
            .get_pipeline(pipeline_id)
            .await
            .map_err(|e| E::PipelineService(e.into()))
    }

    async fn list_snapshots(
        &self,
        pipeline_id: Uuid,
    ) -> Result<Vec<PipelineSnapshotRecord>, JobServiceError> {
        PIPELINE_SNAPSHOT_SERVICE
            .list_snapshots(pipeline_id)
            .await
            .map_err(|e| E::PipelineSnapshotService(e.into()))
    }

    async fn create_snapshot(
        &self,
        pipeline_id: Uuid,
    ) -> Result<PipelineSnapshotRecord, JobServiceError> {
        let snapshot_id = PIPELINE_SNAPSHOT_SERVICE
            .create_snapshot(pipeline_id)
            .await
            .map_err(|e| E::PipelineSnapshotService(e.into()))?;

        PIPELINE_SNAPSHOT_SERVICE
            .get_snapshot(snapshot_id)
            .await
            .map_err(|e| E::PipelineSnapshotService(e.into()))
    }

    pub async fn create_job(
        &self,
        pipeline_id: Uuid,
    ) -> Result<JobCreationResult, JobServiceError> {
        let record = self.get_pipeline(pipeline_id).await?;
        let snapshots = self.list_snapshots(pipeline_id).await?;

        let pipeline_hash = sha2::Sha256::digest(record.content.as_bytes());
        let snapshot_record = match snapshots
            .into_iter()
            .find(|s| sha2::Sha256::digest(s.content.as_bytes()) == pipeline_hash)
        {
            Some(snapshot) => snapshot,
            None => self.create_snapshot(pipeline_id).await?,
        };

        let snapshot_pipeline: Pipeline =
            toml::from_str(&snapshot_record.content).map_err(E::ParsePipeline)?;

        let job = Job::from(snapshot_pipeline);

        let job_id = self
            .job_repo
            .create_job(snapshot_record.id, &job)
            .await
            .map_err(E::JobRepo)?;

        ORCHESTRATOR_SERVICE.queue_job(job).await;

        Ok(JobCreationResult {
            job_id,
            snapshot_id: snapshot_record.id,
        })
    }

    pub async fn update_job(
        &self,
        job_id: Uuid,
        new_status: ExecutionStatus,
    ) -> Result<(), JobServiceError> {
        self.job_repo
            .update_job(
                job_id,
                JobStatusUpdate {
                    status: new_status,
                    updated_at: chrono::Utc::now(),
                },
            )
            .await
            .map_err(E::JobRepo)
    }

    pub async fn update_stage(
        &self,
        stage_id: Uuid,
        new_status: ExecutionStatus,
    ) -> Result<(), JobServiceError> {
        self.job_repo
            .update_stage(
                stage_id,
                StageStatusUpdate {
                    status: new_status,
                    updated_at: chrono::Utc::now(),
                },
            )
            .await
            .map_err(E::JobRepo)
    }

    pub async fn update_step(
        &self,
        step_id: Uuid,
        new_status: ExecutionStatus,
    ) -> Result<(), JobServiceError> {
        self.job_repo
            .update_step(
                step_id,
                StepStatusUpdate {
                    status: new_status,
                    updated_at: chrono::Utc::now(),
                },
            )
            .await
            .map_err(E::JobRepo)
    }
}
