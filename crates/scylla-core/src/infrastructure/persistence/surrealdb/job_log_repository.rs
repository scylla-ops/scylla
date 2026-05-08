use crate::application::ports::JobLogRepository;
use crate::domain::entities::{JobId, JobLog, JobLogId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::pipeline::NodeId;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;
use tracing::instrument;

pub struct SurrealJobLogRepository {
    db: Surreal<Any>,
}

impl SurrealJobLogRepository {
    #[must_use]
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl JobLogRepository for SurrealJobLogRepository {
    #[instrument(skip(self, log), fields(job_log_id = %log.id()))]
    async fn create(&self, log: &JobLog) -> DomainResult<JobLog> {
        let db = self.db.clone();
        let log = log.clone();
        let created: Option<JobLog> = db
            .create(RecordId::new(JobLogId::table_name(), log.id().as_str()))
            .content(log.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        created.ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
    }

    #[instrument(skip(self), fields(job_log_id = %id))]
    async fn find_by_id(&self, id: &JobLogId) -> DomainResult<JobLog> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<JobLog> = db
            .select(RecordId::new(JobLogId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        result.ok_or_else(|| DomainError::not_found("JobLog", id.to_string()))
    }

    #[instrument(skip(self), fields(job_id = %job_id))]
    async fn list_by_job(
        &self,
        job_id: &JobId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        let db = self.db.clone();
        let params = pagination.copied().unwrap_or_default();
        let table = JobLogId::table_name().to_string();
        let job_id = job_id.clone();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE job_id = $job_id GROUP ALL")
            .bind(("table", table.clone()))
            .bind(("job_id", job_id.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let logs: Vec<JobLog> = db
            .query("SELECT * FROM type::table($table) WHERE job_id = $job_id ORDER BY timestamp ASC LIMIT $limit START $start")
            .bind(("table", table))
            .bind(("job_id", job_id.into_value()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(logs, &params, total_count))
    }

    #[instrument(skip(self), fields(job_id = %job_id, node_id = %node_id))]
    async fn list_by_job_and_node(
        &self,
        job_id: &JobId,
        node_id: &NodeId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        let db = self.db.clone();
        let params = pagination.copied().unwrap_or_default();
        let table = JobLogId::table_name().to_string();
        let job_id = job_id.clone();
        let node_id = node_id.clone();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE job_id = $job_id AND node_id = $node_id GROUP ALL")
            .bind(("table", table.clone()))
            .bind(("job_id", job_id.clone().into_value()))
            .bind(("node_id", node_id.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let logs: Vec<JobLog> = db
            .query("SELECT * FROM type::table($table) WHERE job_id = $job_id AND node_id = $node_id ORDER BY timestamp ASC LIMIT $limit START $start")
            .bind(("table", table))
            .bind(("job_id", job_id.into_value()))
            .bind(("node_id", node_id.into_value()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(logs, &params, total_count))
    }

    #[instrument(skip(self), fields(job_id = %job_id, node_id = ?node_id))]
    async fn list_all_by_job(
        &self,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<Vec<JobLog>> {
        let db = self.db.clone();
        let table = JobLogId::table_name().to_string();
        let job_id = job_id.clone();

        let logs: Vec<JobLog> = if let Some(nid) = node_id {
            let nid = nid.clone();
            db.query("SELECT * FROM type::table($table) WHERE job_id = $job_id AND node_id = $node_id ORDER BY timestamp ASC")
                .bind(("table", table))
                .bind(("job_id", job_id.into_value()))
                .bind(("node_id", nid.into_value()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?
        } else {
            db.query("SELECT * FROM type::table($table) WHERE job_id = $job_id ORDER BY timestamp ASC")
                .bind(("table", table))
                .bind(("job_id", job_id.into_value()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?
        };

        Ok(logs)
    }
}
