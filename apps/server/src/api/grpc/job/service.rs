use crate::api::grpc::job::service::JobServiceError as E;
use crate::api::grpc::job::{JobRepository, StageRepository, StepRepository};
use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use crate::api::grpc::pipeline::snapshot::worker::PipelineSnapshotMessage;
use crate::api::grpc::pipeline::worker::PipelineMessage;
use derive_more::Constructor;
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
    stage_repo: Arc<dyn StageRepository>,
    step_repo: Arc<dyn StepRepository>,

    // channel to pipeline service
    tx_pipeline: mpsc::Sender<PipelineMessage>,

    // channel to snapshot service
    tx_pipeline_snapshot: mpsc::Sender<PipelineSnapshotMessage>,
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

    pub async fn create_job(&self, pipeline_id: Uuid) -> Result<(Uuid, Uuid), JobServiceError> {
        let record = self.get_pipeline(pipeline_id).await?;
        let snapshots = self.list_snapshots(pipeline_id).await?;

        let pipeline_hash = format!("{:x}", sha2::Sha256::digest(record.content.as_bytes()));
        let snapshot_record = match snapshots
            .into_iter()
            .find(|s| format!("{:x}", sha2::Sha256::digest(s.content.as_bytes())) == pipeline_hash)
        {
            Some(snapshot) => snapshot,
            None => self.create_snapshot(pipeline_id).await?,
        };

        let pipeline: Pipeline =
            toml::from_str(&snapshot_record.content).map_err(E::ParsePipeline)?;

        //Ok((record.id, record.id));
        unimplemented!()
    }
}
