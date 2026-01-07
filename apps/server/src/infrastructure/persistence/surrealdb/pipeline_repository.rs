use crate::domain::entities::Pipeline;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::PipelineRepository;
use crate::domain::value_objects::PipelineId;
use crate::infrastructure::persistence::mappers::ToRecordId;
use crate::infrastructure::persistence::surrealdb::models::PipelineRecord;
use crate::infrastructure::persistence::{PipelineInsert, PipelineUpdate};
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// SurrealDB implementation of PipelineRepository
#[derive(Constructor)]
pub struct SurrealPipelineRepository {
    db: Arc<Surreal<Any>>,
}

#[async_trait]
impl PipelineRepository for SurrealPipelineRepository {
    async fn create(&self, pipeline: &Pipeline) -> DomainResult<Pipeline> {
        let insert = PipelineInsert::from(pipeline);
        let created: Option<PipelineRecord> = self
            .db
            .create(PipelineId::table_name())
            .content(insert)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match created {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::infrastructure("Failed to create pipeline")),
        }
    }

    async fn find_by_id(&self, id: &PipelineId) -> DomainResult<Pipeline> {
        let result: Option<PipelineRecord> = self
            .db
            .select(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match result {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::not_found("Pipeline", id.to_string())),
        }
    }

    async fn update(&self, pipeline: &Pipeline) -> DomainResult<Pipeline> {
        let record = PipelineUpdate::from(pipeline);
        let updated: Option<PipelineRecord> = self
            .db
            .update(pipeline.id().to_record_id())
            .merge(record)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match updated {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::not_found(
                "Pipeline",
                pipeline.id().to_string(),
            )),
        }
    }

    async fn delete(&self, id: &PipelineId) -> DomainResult<()> {
        self.db
            .delete::<Option<PipelineRecord>>(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<Pipeline>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM type::table($table) GROUP ALL")
            .bind(("table", PipelineId::table_name()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Get paginated records
        let records: Vec<PipelineRecord> = self
            .db
            .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("table", PipelineId::table_name()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let pipelines: DomainResult<Vec<Pipeline>> =
            records.into_iter().map(TryFrom::try_from).collect();

        Ok(PaginatedResult::new(pipelines?, &params, total_count))
    }
}
