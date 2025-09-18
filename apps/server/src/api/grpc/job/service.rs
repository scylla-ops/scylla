use crate::api::grpc::job::JobRepository;
use crate::api::grpc::job::models::{
    ExecutionStatus, JobStatusUpdate, StageStatusUpdate, StepStatusUpdate,
};
use crate::api::grpc::job::service::JobServiceError as E;
use crate::api::grpc::orchestrator::worker::OrchestratorMessage;
use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use crate::api::grpc::pipeline::snapshot::worker::PipelineSnapshotMessage;
use crate::api::grpc::pipeline::worker::PipelineMessage;
use derive_more::Constructor;
use protocol::job::Job;
use protocol::pipeline::Pipeline;
use protocol::toml;
use sha2::Digest;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

#[derive(Constructor)]
pub struct JobService {
    job_repo: Arc<dyn JobRepository>,

    // channel to pipeline service
    tx_pipeline: mpsc::Sender<PipelineMessage>,

    // channel to snapshot service
    tx_pipeline_snapshot: mpsc::Sender<PipelineSnapshotMessage>,

    // channel to orchestrator
    tx_orchestrator: mpsc::Sender<OrchestratorMessage>,
}

#[derive(Debug, Error)]
pub enum JobServiceError {
    #[error("Failed to parse pipeline: {0}")]
    ParsePipeline(#[from] toml::de::Error),
    #[error("Internal: unable to use channel {0}")]
    Channel(anyhow::Error),
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
        let (tx, rx) = oneshot::channel();
        self.tx_pipeline
            .send(PipelineMessage::GetPipeline {
                id: pipeline_id,
                respond_tx: tx,
            })
            .await
            .map_err(|e| E::Channel(e.into()))?;

        rx.await
            .map_err(|e| E::Channel(e.into()))?
            .map_err(E::PipelineService)
    }

    async fn list_snapshots(
        &self,
        pipeline_id: Uuid,
    ) -> Result<Vec<PipelineSnapshotRecord>, JobServiceError> {
        let (tx, rx) = oneshot::channel();
        self.tx_pipeline_snapshot
            .send(PipelineSnapshotMessage::ListSnapshots { pipeline_id, tx })
            .await
            .map_err(|e| E::Channel(e.into()))?;

        rx.await
            .map_err(|e| E::Channel(e.into()))?
            .map_err(E::PipelineSnapshotService)
    }

    async fn create_snapshot(
        &self,
        pipeline_id: Uuid,
    ) -> Result<PipelineSnapshotRecord, JobServiceError> {
        let (tx, rx) = oneshot::channel();
        self.tx_pipeline_snapshot
            .send(PipelineSnapshotMessage::CreateSnapshot { pipeline_id, tx })
            .await
            .map_err(|e| E::Channel(e.into()))?;

        rx.await
            .map_err(|e| E::Channel(e.into()))?
            .map_err(E::PipelineSnapshotService)
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

        self.tx_orchestrator
            .send(OrchestratorMessage::NewJob { job: job.clone() })
            .await
            .map_err(|e| E::Channel(e.into()))?;

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
