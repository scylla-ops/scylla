use crate::api::base::BaseRepository;
use crate::api::base::diesel_repo_base::Repository;
use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::api::grpc::pipeline::snapshot::PipelineSnapshotRepository;
use crate::api::grpc::pipeline::snapshot::models::PipelineSnapshotRecord;
use crate::database::DieselPool;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::{BelongingToDsl, ExpressionMethods, SelectableHelper};
use repository_derive::DieselRepository;
use uuid::Uuid;

#[derive(DieselRepository)]
pub struct PipelineSnapshotRepositoryDiesel {
    base: BaseRepository,
}

#[async_trait::async_trait]
impl PipelineSnapshotRepository for PipelineSnapshotRepositoryDiesel {
    async fn create_snapshot(&self, pipeline: PipelineRecord) -> anyhow::Result<Uuid> {
        use crate::database::schema::pipeline_snapshots::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let new_pipeline_snapshot = super::models::NewPipelineSnapshot {
            pipeline_id: pipeline.id,
            content: &pipeline.content,
        };

        let snapshot_id = diesel::insert_into(pipeline_snapshots)
            .values(&new_pipeline_snapshot)
            .returning(id)
            .get_result::<Uuid>(&mut conn)?;

        Ok(snapshot_id)
    }

    async fn get_snapshot(&self, snapshot_id: Uuid) -> anyhow::Result<PipelineSnapshotRecord> {
        use crate::database::schema::pipeline_snapshots::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let record: PipelineSnapshotRecord = pipeline_snapshots
            .filter(id.eq(snapshot_id))
            .first(&mut conn)?;

        Ok(record)
    }

    async fn list_snapshots(
        &self,
        pipeline: PipelineRecord,
    ) -> anyhow::Result<Vec<PipelineSnapshotRecord>> {
        let mut conn = Repository::get_connection(self)?;

        let records = PipelineSnapshotRecord::belonging_to(&pipeline)
            .select(PipelineSnapshotRecord::as_select())
            .load(&mut conn)?;

        Ok(records)
    }

    async fn delete_snapshot(&self, snapshot_id: Uuid) -> anyhow::Result<()> {
        use crate::database::schema::pipeline_snapshots::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        diesel::delete(pipeline_snapshots)
            .filter(id.eq(snapshot_id))
            .execute(&mut conn)?;

        Ok(())
    }
}
