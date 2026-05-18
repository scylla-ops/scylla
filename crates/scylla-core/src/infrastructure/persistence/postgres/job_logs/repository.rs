use crate::application::ports::JobLogRepository;
use crate::domain::entities::{JobId, JobLog, JobLogId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::job::LogStream;
use crate::domain::value_objects::pipeline::NodeId;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};

#[derive(Clone)]
pub struct PgJobLogRepository {
    pool: PgPool,
}

impl PgJobLogRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobLogRepository for PgJobLogRepository {
    #[instrument(skip(self, log), fields(job_id = %log.job_id()))]
    async fn create(&self, log: &JobLog) -> DomainResult<JobLog> {
        queries::create(&self.pool, log).await
    }

    #[instrument(skip(self), fields(log_id = %id))]
    async fn find_by_id(&self, id: &JobLogId) -> DomainResult<JobLog> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip(self), fields(job_id = %job_id))]
    async fn list_by_job(
        &self,
        job_id: &JobId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_by_job(&self.pool, job_id, None).await?;
        let items = queries::list_page(&self.pool, job_id, None, &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(job_id = %job_id, node_id = %node_id))]
    async fn list_by_job_and_node(
        &self,
        job_id: &JobId,
        node_id: &NodeId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<JobLog>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_by_job(&self.pool, job_id, Some(node_id)).await?;
        let items = queries::list_page(&self.pool, job_id, Some(node_id), &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(job_id = %job_id))]
    async fn list_all_by_job(
        &self,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<Vec<JobLog>> {
        queries::list_all(&self.pool, job_id, node_id).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    fn row_into_log(
        id: String,
        job_id: String,
        node_id: String,
        stream: String,
        line: String,
        timestamp: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> DomainResult<JobLog> {
        let node_id = NodeId::new(node_id).db_field("node_id")?;
        let stream = LogStream::new(stream).db_field("log stream")?;
        Ok(JobLog::from_persistence(
            JobLogId::new(id),
            JobId::new(job_id),
            node_id,
            stream,
            line,
            timestamp,
            created_at,
        ))
    }

    pub async fn create<'e, E>(executor: E, log: &JobLog) -> DomainResult<JobLog>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO job_logs (id, job_id, node_id, stream, line, timestamp, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            log.id().as_str(),
            log.job_id().as_str(),
            log.node_id().as_str(),
            log.stream().as_str(),
            log.line(),
            log.timestamp(),
            log.created_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(log.clone())
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &JobLogId) -> DomainResult<JobLog>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, job_id, node_id, stream, line, timestamp, created_at
            FROM job_logs
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("JobLog", id.to_string())?;
        row_into_log(
            rec.id,
            rec.job_id,
            rec.node_id,
            rec.stream,
            rec.line,
            rec.timestamp,
            rec.created_at,
        )
    }

    pub async fn count_by_job<'e, E>(
        executor: E,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM job_logs
            WHERE job_id = $1
              AND ($2::text IS NULL OR node_id = $2)
            "#,
            job_id.as_str(),
            node_id.map(AsRef::as_ref),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_page<'e, E>(
        executor: E,
        job_id: &JobId,
        node_id: Option<&NodeId>,
        params: &PaginationParams,
    ) -> DomainResult<Vec<JobLog>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows = sqlx::query!(
            r#"
            SELECT id, job_id, node_id, stream, line, timestamp, created_at
            FROM job_logs
            WHERE job_id = $1
              AND ($2::text IS NULL OR node_id = $2)
            ORDER BY timestamp ASC, id ASC
            LIMIT $3 OFFSET $4
            "#,
            job_id.as_str(),
            node_id.map(AsRef::as_ref),
            limit,
            offset,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter()
            .map(|r| {
                row_into_log(
                    r.id,
                    r.job_id,
                    r.node_id,
                    r.stream,
                    r.line,
                    r.timestamp,
                    r.created_at,
                )
            })
            .collect()
    }

    pub async fn list_all<'e, E>(
        executor: E,
        job_id: &JobId,
        node_id: Option<&NodeId>,
    ) -> DomainResult<Vec<JobLog>>
    where
        E: PgExecutor<'e>,
    {
        let rows = sqlx::query!(
            r#"
            SELECT id, job_id, node_id, stream, line, timestamp, created_at
            FROM job_logs
            WHERE job_id = $1
              AND ($2::text IS NULL OR node_id = $2)
            ORDER BY timestamp ASC, id ASC
            "#,
            job_id.as_str(),
            node_id.map(AsRef::as_ref),
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter()
            .map(|r| {
                row_into_log(
                    r.id,
                    r.job_id,
                    r.node_id,
                    r.stream,
                    r.line,
                    r.timestamp,
                    r.created_at,
                )
            })
            .collect()
    }
}
