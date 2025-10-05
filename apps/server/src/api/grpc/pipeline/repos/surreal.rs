use crate::api::grpc::pipeline::models::{NewPipeline, PipelinePatch, PipelineRecord};
use crate::api::grpc::pipeline::repos::{PipelineRepository, TABLE};
use crate::api::grpc::utils::Id;
use crate::database::db;
use anyhow::Context;
use async_trait::async_trait;

#[derive(Default)]
pub struct PipelineRepositorySurreal;

#[async_trait]
impl PipelineRepository for PipelineRepositorySurreal {
    async fn create_pipeline(new_pipeline: NewPipeline) -> anyhow::Result<Id> {
        let rec: Option<PipelineRecord> = db()
            .create(TABLE)
            .content(new_pipeline)
            .await
            .context("Failed to create pipeline")?;

        let row = rec.context("Failed to fetch pipeline")?;
        Ok(row.id.key().to_string())
    }

    async fn get_pipeline(id: Id) -> anyhow::Result<PipelineRecord> {
        let rec: Option<PipelineRecord> = db()
            .select((TABLE, &id))
            .await
            .context("Failed to get pipeline")?;

        rec.context("Pipeline not found")
    }

    async fn list_pipelines() -> anyhow::Result<Vec<PipelineRecord>> {
        let records: Vec<PipelineRecord> = db()
            .select(TABLE)
            .await
            .context("Failed to list pipelines")?;

        Ok(records)
    }

    async fn delete_pipeline(id: Id) -> anyhow::Result<()> {
        let rec: Option<PipelineRecord> = db()
            .delete((TABLE, &id))
            .await
            .context("Failed to delete pipeline")?;

        rec.context("Pipeline not found")?;
        Ok(())
    }

    async fn update_pipeline(id: Id, patch: PipelinePatch) -> anyhow::Result<()> {
        let rec: Option<PipelineRecord> = db()
            .update((TABLE, &id))
            .merge(patch)
            .await
            .context("Failed to update pipeline")?;

        rec.context("Pipeline not found")?;
        Ok(())
    }
}
