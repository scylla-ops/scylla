use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::service::PIPELINE_SERVICE;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use crate::api::grpc::pipeline::snapshot::repo::PipelineSnapshotRepositoryDiesel;
use crate::api::grpc::pipeline::snapshot::PipelineSnapshotRepository;
use crate::database::get_existing_db;
use derive_more::Constructor;
use std::sync::{Arc, LazyLock};
use thiserror::Error;
use uuid::Uuid;

#[derive(Constructor)]
pub struct PipelineSnapshotService {
    repo: Arc<dyn PipelineSnapshotRepository>,
}

#[derive(Debug, Error)]
pub enum PipelineSnapshotServiceError {
    #[error("Pipeline service error: {0}")]
    PipelineServiceError(anyhow::Error),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

pub static PIPELINE_SNAPSHOT_SERVICE: LazyLock<Arc<PipelineSnapshotService>> =
    LazyLock::new(|| {
        let diesel_db = get_existing_db();

        Arc::new(PipelineSnapshotService::new(Arc::new(
            PipelineSnapshotRepositoryDiesel::new(diesel_db.clone()),
        )))
    });

impl PipelineSnapshotService {
    pub async fn create_snapshot(
        &self,
        pipeline_id: Uuid,
    ) -> Result<Uuid, PipelineSnapshotServiceError> {
        let pipeline_rec: PipelineRecord = PIPELINE_SERVICE
            .get_pipeline(pipeline_id)
            .await
            .map_err(|e| PipelineSnapshotServiceError::PipelineServiceError(e.into()))?;

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
        let pipeline_rec: PipelineRecord = PIPELINE_SERVICE
            .get_pipeline(pipeline_id)
            .await
            .map_err(|e| PipelineSnapshotServiceError::PipelineServiceError(e.into()))?;

        let records = self
            .repo
            .list_snapshots(pipeline_rec)
            .await
            .map_err(PipelineSnapshotServiceError::Repo)?;
        Ok(records)
    }
}
