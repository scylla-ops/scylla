use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::snapshot::PipelineSnapshotRepository;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use crate::api::grpc::pipeline::worker::PipelineMessage;
use derive_more::Constructor;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Constructor)]
pub struct PipelineSnapshotService {
    repo: Arc<dyn PipelineSnapshotRepository>,
    tx_pipeline: mpsc::Sender<PipelineMessage>,
}

#[derive(Debug, Error)]
pub enum PipelineSnapshotError {
    #[error("Critical: unable to send GetPipeline request")]
    SendFailed,
    #[error("Critical: unable to receive pipeline")]
    ReceiveFailed,
    #[error("Pipeline not found")]
    PipelineNotFound,
    #[error("Create failed: {0}")]
    CreateFailed(anyhow::Error),
    #[error("Get failed: {0}")]
    GetFailed(anyhow::Error),
    #[error("Delete failed: {0}")]
    DeleteFailed(anyhow::Error),
    #[error("List failed: {0}")]
    ListFailed(anyhow::Error),
}

impl PipelineSnapshotService {
    pub async fn create_snapshot(&self, pipeline_id: Uuid) -> Result<Uuid, PipelineSnapshotError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx_pipeline
            .send(PipelineMessage::GetPipeline {
                id: pipeline_id,
                respond_tx: tx,
            })
            .await
            .map_err(|_| PipelineSnapshotError::SendFailed)?;

        let pipeline_rec: PipelineRecord = rx
            .await
            .map_err(|_| PipelineSnapshotError::ReceiveFailed)?
            .map_err(|e| {
                tracing::error!("Failed to retrieve pipeline: {}", e);
                PipelineSnapshotError::PipelineNotFound
            })?;

        let snapshot_id = self
            .repo
            .create_snapshot(pipeline_rec)
            .await
            .map_err(PipelineSnapshotError::CreateFailed)?;

        Ok(snapshot_id)
    }

    pub async fn get_snapshot(
        &self,
        snapshot_id: Uuid,
    ) -> Result<PipelineSnapshotRecord, PipelineSnapshotError> {
        let record = self
            .repo
            .get_snapshot(snapshot_id)
            .await
            .map_err(PipelineSnapshotError::GetFailed)?;
        Ok(record)
    }

    pub async fn delete_snapshot(&self, snapshot_id: Uuid) -> Result<(), PipelineSnapshotError> {
        self.repo
            .delete_snapshot(snapshot_id)
            .await
            .map_err(PipelineSnapshotError::DeleteFailed)?;
        Ok(())
    }

    pub async fn list_snapshots(
        &self,
        pipeline_id: Uuid,
    ) -> Result<Vec<PipelineSnapshotRecord>, PipelineSnapshotError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx_pipeline
            .send(PipelineMessage::GetPipeline {
                id: pipeline_id,
                respond_tx: tx,
            })
            .await
            .map_err(|_| PipelineSnapshotError::SendFailed)?;

        let pipeline_rec: PipelineRecord = rx
            .await
            .map_err(|_| PipelineSnapshotError::ReceiveFailed)?
            .map_err(|e| {
                tracing::error!("Failed to retrieve pipeline: {}", e);
                PipelineSnapshotError::PipelineNotFound
            })?;

        let records = self
            .repo
            .list_snapshots(pipeline_rec)
            .await
            .map_err(PipelineSnapshotError::ListFailed)?;
        Ok(records)
    }
}
