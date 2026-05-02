use crate::application::ports::JobLogRepository;
use crate::application::ports::repositories::job_log_repo::JobLogStream;
use crate::domain::entities::{JobId, JobLog, JobLogId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::pipeline::NodeId;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use futures_util::StreamExt;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb_types::Action;
use surrealdb_types::SurrealValue;
use tracing::{info, instrument, warn};

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
    async fn watch(
        &self,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<JobLogStream> {
        let db = self.db.clone();
        let table = JobLogId::table_name().to_string();
        let target_job_id = job_id.clone();
        let target_node_id = node_id.cloned();

        info!(table = %table, "opening live select via .select().live() — probing with Value");
        // Probe stage: deserialize as raw surrealdb_types::Value to confirm we receive notifications at all.
        let stream = db
            .select::<Vec<surrealdb_types::Value>>(table.as_str())
            .live()
            .await
            .map_err(|e| {
                warn!("Live query open error: {e}");
                DomainError::infrastructure(format!("Live query error: {e}"))
            })?;
        info!("live select opened, awaiting notifications (Value probe)");

        let filtered = stream.filter_map(move |item: Result<surrealdb::Notification<surrealdb_types::Value>, surrealdb::Error>| {
            let target_job_id = target_job_id.clone();
            let target_node_id = target_node_id.clone();
            let _ = (&target_job_id, &target_node_id);
            async move {
                match item {
                    Ok(notif) => {
                        info!(action = ?notif.action, data = ?notif.data, "live notification received (Value probe)");
                        // Try deserialize the Value into JobLog
                        match notif.action {
                            Action::Create => {
                                match <JobLog as SurrealValue>::from_value(notif.data) {
                                    Ok(log) => {
                                        info!(
                                            log_job_id = %log.job_id(),
                                            log_node_id = %log.node_id(),
                                            target_job_id = %target_job_id,
                                            "filtering log entry"
                                        );
                                        if log.job_id() != &target_job_id {
                                            info!("dropped: job_id mismatch");
                                            return None;
                                        }
                                        if let Some(ref nid) = target_node_id {
                                            if log.node_id() != nid {
                                                info!("dropped: node_id mismatch");
                                                return None;
                                            }
                                        }
                                        info!("forwarding log");
                                        Some(Ok(log))
                                    }
                                    Err(e) => {
                                        warn!("JobLog::from_value error: {e}");
                                        None
                                    }
                                }
                            }
                            other => {
                                info!(action = ?other, "ignoring non-create action");
                                None
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Live query notification error: {e}");
                        Some(Err(DomainError::infrastructure(format!(
                            "Live query notification error: {e}"
                        ))))
                    }
                }
            }
        });

        Ok(Box::pin(filtered))
    }
}
