use crate::domain::entities::Job;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::JobRepository;
use crate::domain::value_objects::JobStatus;
use crate::domain::value_objects::{JobId, PipelineId};
use crate::infrastructure::persistence::surrealdb::mappers::ToRecordId;
use crate::infrastructure::persistence::surrealdb::models::JobRecord;

use crate::infrastructure::persistence::{JobInsert, JobUpdate};
use async_trait::async_trait;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// SurrealDB implementation of JobRepository
pub struct SurrealJobRepository {
    db: Arc<Surreal<Any>>,
}

impl SurrealJobRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl JobRepository for SurrealJobRepository {
    async fn create(&self, job: &Job) -> DomainResult<Job> {
        let insert = JobInsert::from(job);

        let created: Option<JobRecord> = self
            .db
            .create("jobs")
            .content(insert)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match created {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::infrastructure("Failed to create job")),
        }
    }

    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job> {
        let result: Option<JobRecord> = self
            .db
            .select(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match result {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::not_found("Job", id.to_string())),
        }
    }

    async fn update(&self, job: &Job) -> DomainResult<Job> {
        let update = JobUpdate::from(job);

        let updated: Option<JobRecord> = self
            .db
            .update(job.id().to_record_id())
            .merge(update)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match updated {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::infrastructure("Failed to update job")),
        }
    }

    async fn delete(&self, id: &JobId) -> DomainResult<()> {
        self.db
            .delete::<Option<JobRecord>>(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<Job>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM jobs GROUP ALL")
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
        let records: Vec<JobRecord> = self
            .db
            .query("SELECT * FROM jobs ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let jobs: DomainResult<Vec<Job>> = records
            .into_iter()
            .map(|record| record.try_into())
            .collect();

        Ok(PaginatedResult::new(jobs?, &params, total_count))
    }

    async fn list_by_status(
        &self,
        status: &JobStatus,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<Job>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);
        let status_str = status.as_str();

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM jobs WHERE status = $status GROUP ALL")
            .bind(("status", status_str))
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
        let records: Vec<JobRecord> = self
			.db
			.query("SELECT * FROM jobs WHERE status = $status ORDER BY created_at DESC LIMIT $limit START $start")
			.bind(("status", status_str))
			.bind(("limit", params.limit()))
			.bind(("start", params.offset()))
			.await
			.map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
			.take(0)
			.map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let jobs: DomainResult<Vec<Job>> = records
            .into_iter()
            .map(|record| record.try_into())
            .collect();

        Ok(PaginatedResult::new(jobs?, &params, total_count))
    }

    async fn list_by_pipeline(
        &self,
        pipeline_id: &PipelineId,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<Job>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM jobs WHERE pipeline_id = $pipeline_id GROUP ALL")
            .bind(("pipeline_id", PipelineId::to_record_id(pipeline_id)))
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
        let records: Vec<JobRecord> = self
			.db
			.query("SELECT * FROM jobs WHERE pipeline_id = $pipeline_id ORDER BY created_at DESC LIMIT $limit START $start")
			.bind(("pipeline_id", PipelineId::to_record_id(pipeline_id)))
			.bind(("limit", params.limit()))
			.bind(("start", params.offset()))
			.await
			.map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
			.take(0)
			.map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let jobs: DomainResult<Vec<Job>> = records
            .into_iter()
            .map(|record| record.try_into())
            .collect();

        Ok(PaginatedResult::new(jobs?, &params, total_count))
    }
}
