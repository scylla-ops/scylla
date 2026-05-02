use crate::application::ports::JobLogRepository;
use crate::application::ports::repositories::job_log_repo::JobLogStream;
use crate::domain::entities::{JobId, JobLog, JobLogId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::pipeline::NodeId;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
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
        use futures_util::stream::StreamExt as _;

        let db = self.db.clone();
        let table = JobLogId::table_name().to_string();
        let target_job_id = job_id.clone();
        let target_node_id = node_id.cloned();

        let (tx, rx) = tokio::sync::mpsc::channel::<DomainResult<JobLog>>(256);

        // The SurrealDB SDK live() subscription's waker routing is tied to the
        // task that awaits `.live()`. To keep `.live().await` and `.next().await`
        // on the same task, we open and drive the stream from a dedicated
        // tokio task and forward filtered results through an mpsc channel.
        tokio::spawn(async move {
            info!(table = %table, "live select: opening from spawned task");
            let mut stream = match db
                .select::<Vec<surrealdb_types::Value>>(table.as_str())
                .live()
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    warn!("Live query open error: {e}");
                    let _ = tx
                        .send(Err(DomainError::infrastructure(format!(
                            "Live query error: {e}"
                        ))))
                        .await;
                    return;
                }
            };
            info!("live select opened, forwarding notifications");

            while let Some(item) = stream.next().await {
                if tx.is_closed() {
                    break;
                }
                match item {
                    Ok(notif) => match notif.action {
                        Action::Create => {
                            match <JobLog as SurrealValue>::from_value(notif.data) {
                                Ok(log) => {
                                    if log.job_id() != &target_job_id {
                                        continue;
                                    }
                                    if let Some(ref nid) = target_node_id {
                                        if log.node_id() != nid {
                                            continue;
                                        }
                                    }
                                    if tx.send(Ok(log)).await.is_err() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    warn!("JobLog::from_value error: {e}");
                                }
                            }
                        }
                        _ => {}
                    },
                    Err(e) => {
                        warn!("Live query notification error: {e}");
                        let _ = tx
                            .send(Err(DomainError::infrastructure(format!(
                                "Live query notification error: {e}"
                            ))))
                            .await;
                    }
                }
            }
            info!("live forwarder task ended");
        });

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
