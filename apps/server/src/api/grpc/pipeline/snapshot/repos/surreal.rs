use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::snapshot::models::{NewPipelineSnapshot, PipelineSnapshotRecord};
use crate::api::grpc::pipeline::snapshot::repos::{PipelineSnapshotRepository, TABLE};
use crate::api::grpc::utils::Id;
use crate::database::db;
use anyhow::Context;
use async_trait::async_trait;

pub struct PipelineSnapshotRepositorySurreal;

#[async_trait]
impl PipelineSnapshotRepository for PipelineSnapshotRepositorySurreal {
    async fn create_snapshot(new_snapshot: NewPipelineSnapshot) -> anyhow::Result<Id> {
        let rec: Option<PipelineSnapshotRecord> = db()
            .create(TABLE)
            .content(new_snapshot)
            .await
            .context("Failed to execute create snapshot query")?;

        Ok(rec
            .context("Failed to fetch snapshot")?
            .id
            .key()
            .to_string())
    }

    async fn get_snapshot(id: Id) -> anyhow::Result<PipelineSnapshotRecord> {
        let rec: Option<PipelineSnapshotRecord> = db().select((TABLE, id)).await?;
        rec.context("Failed to fetch snapshot")
    }

    async fn list_snapshots(
        pipeline: &PipelineRecord,
    ) -> anyhow::Result<Vec<PipelineSnapshotRecord>> {
        let mut resp = db()
            .query(format!(
                "SELECT * FROM {} WHERE pipeline = {}",
                TABLE, pipeline.id
            ))
            .await
            .context("Failed to execute list snapshots query")?;

        let records: Vec<PipelineSnapshotRecord> = resp.take(0)?;
        Ok(records)
    }

    async fn delete_snapshot(id: Id) -> anyhow::Result<()> {
        let rec: Option<PipelineSnapshotRecord> = db()
            .delete((TABLE, &id))
            .await
            .context("Failed to delete snapshot")?;

        rec.context("Snapshot not found")?;
        Ok(())
    }
}
