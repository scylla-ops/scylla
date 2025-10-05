use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::repos::PipelineRepository;
use crate::api::grpc::pipeline::snapshot::models::{NewPipelineSnapshot, PipelineSnapshotRecord};
use crate::api::grpc::pipeline::snapshot::repos::PipelineSnapshotRepository;
use crate::api::grpc::utils::Id;
use derive_more::Constructor;
use thiserror::Error;

#[derive(Constructor)]
pub struct PipelineSnapshotService<SR: PipelineSnapshotRepository, PR: PipelineRepository> {
    _marker: std::marker::PhantomData<(SR, PR)>,
}

#[derive(Debug, Error)]
pub enum PipelineSnapshotServiceError {
    #[error("Pipeline service error: {0}")]
    PipelineServiceError(anyhow::Error),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

impl<SR: PipelineSnapshotRepository, PR: PipelineRepository> PipelineSnapshotService<SR, PR> {
    pub async fn create_snapshot(pipeline_id: Id) -> Result<Id, PipelineSnapshotServiceError> {
        let pipeline_rec: PipelineRecord = PR::get_pipeline(pipeline_id)
            .await
            .map_err(PipelineSnapshotServiceError::PipelineServiceError)?;

        let snapshot_id = SR::create_snapshot(NewPipelineSnapshot {
            pipeline: pipeline_rec.id,
            content: pipeline_rec.content,
        })
        .await?;

        Ok(snapshot_id)
    }

    pub async fn get_snapshot(
        snapshot_id: Id,
    ) -> Result<PipelineSnapshotRecord, PipelineSnapshotServiceError> {
        let record = SR::get_snapshot(snapshot_id).await?;
        Ok(record)
    }

    pub async fn delete_snapshot(snapshot_id: Id) -> Result<(), PipelineSnapshotServiceError> {
        SR::delete_snapshot(snapshot_id).await?;
        Ok(())
    }

    pub async fn list_snapshots(
        pipeline_id: Id,
    ) -> Result<Vec<PipelineSnapshotRecord>, PipelineSnapshotServiceError> {
        let pipeline_rec: PipelineRecord = PR::get_pipeline(pipeline_id)
            .await
            .map_err(PipelineSnapshotServiceError::PipelineServiceError)?;

        let records = SR::list_snapshots(&pipeline_rec).await?;
        Ok(records)
    }
}
