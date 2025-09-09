use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::snapshot::PipelineSnapshotRepository;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use crate::api::grpc::pipeline::worker::PipelineMessage;
use derive_more::Constructor;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::oneshot::Receiver;
use uuid::Uuid;

#[derive(Constructor)]
pub struct PipelineSnapshotService {
    repo: Arc<dyn PipelineSnapshotRepository>,
    tx_pipeline: mpsc::Sender<PipelineMessage>,
}

#[derive(Debug, Error)]
pub enum PipelineSnapshotServiceError {
    #[error("Internal: unable to use channel {0}")]
    ChannelError(anyhow::Error),
    #[error("Pipeline service error: {0}")]
    PipelineServiceError(anyhow::Error),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

impl PipelineSnapshotService {
    pub async fn create_snapshot(
        &self,
        pipeline_id: Uuid,
    ) -> Result<Uuid, PipelineSnapshotServiceError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx_pipeline
            .send(PipelineMessage::GetPipeline {
                id: pipeline_id,
                respond_tx: tx,
            })
            .await
            .map_err(|e| PipelineSnapshotServiceError::ChannelError(e.into()))?;

        let pipeline_rec: PipelineRecord = self.get_pipeline_record(rx).await?;

        let snapshot_id = self
            .repo
            .create_snapshot(pipeline_rec)
            .await
            .map_err(PipelineSnapshotServiceError::Repo)?;

        Ok(snapshot_id)
    }

    pub async fn get_snapshot(
        &self,
        snapshot_id: Uuid,
    ) -> Result<PipelineSnapshotRecord, PipelineSnapshotServiceError> {
        let record = self
            .repo
            .get_snapshot(snapshot_id)
            .await
            .map_err(PipelineSnapshotServiceError::Repo)?;
        Ok(record)
    }

    pub async fn delete_snapshot(
        &self,
        snapshot_id: Uuid,
    ) -> Result<(), PipelineSnapshotServiceError> {
        self.repo
            .delete_snapshot(snapshot_id)
            .await
            .map_err(PipelineSnapshotServiceError::Repo)?;
        Ok(())
    }

    pub async fn list_snapshots(
        &self,
        pipeline_id: Uuid,
    ) -> Result<Vec<PipelineSnapshotRecord>, PipelineSnapshotServiceError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx_pipeline
            .send(PipelineMessage::GetPipeline {
                id: pipeline_id,
                respond_tx: tx,
            })
            .await
            .map_err(|e| PipelineSnapshotServiceError::ChannelError(e.into()))?;

        let pipeline_rec: PipelineRecord = self.get_pipeline_record(rx).await?;

        let records = self
            .repo
            .list_snapshots(pipeline_rec)
            .await
            .map_err(PipelineSnapshotServiceError::Repo)?;
        Ok(records)
    }

    async fn get_pipeline_record(
        &self,
        rx: Receiver<anyhow::Result<PipelineRecord>>,
    ) -> Result<PipelineRecord, PipelineSnapshotServiceError> {
        rx.await
            .map_err(|e| PipelineSnapshotServiceError::ChannelError(e.into()))?
            .map_err(|e| {
                tracing::error!("Failed to retrieve pipeline: {}", e);
                PipelineSnapshotServiceError::PipelineServiceError(e.into())
            })
    }
}
