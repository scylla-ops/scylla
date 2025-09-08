use crate::api::base::BaseRepository;
use crate::api::base::diesel_repo_base::Repository;
use crate::api::grpc::pipeline::PipelineRepository;
use crate::api::grpc::pipeline::models::PipelineRecord;
use crate::database::DieselPool;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use protocol::pipeline::Pipeline;
use protocol::toml;
use repository_derive::DieselRepository;
use uuid::Uuid;

#[derive(DieselRepository, Clone)]
pub struct PipelineRepositoryDiesel {
    base: BaseRepository,
}

#[async_trait::async_trait]
impl PipelineRepository for PipelineRepositoryDiesel {
    async fn create_pipeline(&self, pipeline: Pipeline) -> anyhow::Result<Uuid> {
        use crate::database::schema::pipelines::dsl::*;

        let new_pipeline = super::models::NewPipeline {
            content: &toml::to_string(&pipeline)?,
        };

        let mut conn = Repository::get_connection(self)?;

        let inserted_id: Uuid = diesel::insert_into(pipelines)
            .values(&new_pipeline)
            .returning(id)
            .get_result(&mut conn)?;

        Ok(inserted_id)
    }

    async fn get_pipeline(&self, pipeline_id: Uuid) -> anyhow::Result<PipelineRecord> {
        use crate::database::schema::pipelines::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let record: PipelineRecord = pipelines.filter(id.eq(pipeline_id)).first(&mut conn)?;

        Ok(record)
    }

    async fn list_pipelines(&self) -> anyhow::Result<Vec<Pipeline>> {
        todo!()
    }

    async fn delete_pipeline(&self, pipeline_id: Uuid) -> anyhow::Result<()> {
        use crate::database::schema::pipelines::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let deleted_row =
            diesel::delete(pipelines.filter(id.eq(pipeline_id))).execute(&mut conn)?;

        match deleted_row {
            1 => Ok(()),
            _ => Err(anyhow::anyhow!(format!(
                "Pipeline with id {} was not found",
                pipeline_id
            ))),
        }
    }

    async fn update_pipeline(
        &self,
        pipeline_id: Uuid,
        updated_pipeline: Pipeline,
    ) -> anyhow::Result<()> {
        use crate::database::schema::pipelines::dsl::*;
        use chrono::Utc;
        let mut conn = Repository::get_connection(self)?;

        let updated_content = toml::to_string(&updated_pipeline)?;
        let now = Utc::now().naive_utc();

        let affected_rows = diesel::update(pipelines.filter(id.eq(pipeline_id)))
            .set((content.eq(updated_content), updated_at.eq(now)))
            .execute(&mut conn)?;

        match affected_rows {
            1 => Ok(()),
            0 => Err(anyhow::anyhow!(format!(
                "Pipeline with id {} was not found",
                pipeline_id
            ))),
            _ => Err(anyhow::anyhow!(
                "Error updating pipeline with id {}",
                pipeline_id
            )),
        }
    }
}
