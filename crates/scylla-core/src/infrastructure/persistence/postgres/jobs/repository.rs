use crate::application::JobRepository;
use crate::domain::entities::{AppId, Job, JobId, JobNode, OrganizationId, PipelineId, ProjectId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::job::{JobOrigin, JobStatus};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool, types::Json};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};

#[derive(Clone)]
pub struct PgJobRepository {
    pool: PgPool,
}

impl PgJobRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobRepository for PgJobRepository {
    #[instrument(skip(self, job), fields(job_id = %job.id()))]
    async fn create(&self, job: &Job) -> DomainResult<Job> {
        queries::create(&self.pool, job).await
    }

    #[instrument(skip(self), fields(job_id = %id))]
    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip(self, job), fields(job_id = %job.id()))]
    async fn update(&self, job: &Job) -> DomainResult<Job> {
        queries::update(&self.pool, job).await
    }

    #[instrument(skip(self), fields(job_id = %job_id, app_id = %app_id))]
    async fn set_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()> {
        queries::set_agent(&self.pool, job_id, app_id).await
    }

    #[instrument(skip(self))]
    async fn list_pending_unassigned(&self) -> DomainResult<Vec<Job>> {
        queries::list_pending_unassigned(&self.pool).await
    }

    #[instrument(skip(self), fields(job_id = %id))]
    async fn delete(&self, id: &JobId) -> DomainResult<()> {
        queries::delete(&self.pool, id).await
    }

    #[instrument(skip(self))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count(&self.pool, Scope::All).await?;
        let items = queries::list_page(&self.pool, &params, Scope::All).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(pipeline_id = %pipeline_id))]
    async fn list_by_pipeline(
        &self,
        pipeline_id: &PipelineId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count(&self.pool, Scope::Pipeline(pipeline_id)).await?;
        let items = queries::list_page(&self.pool, &params, Scope::Pipeline(pipeline_id)).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(project_id = %project_id))]
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count(&self.pool, Scope::Project(project_id)).await?;
        let items = queries::list_page(&self.pool, &params, Scope::Project(project_id)).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(org_id = %organization_id))]
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count(&self.pool, Scope::Organization(organization_id)).await?;
        let items =
            queries::list_page(&self.pool, &params, Scope::Organization(organization_id)).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }
}

#[derive(Clone, Copy)]
pub enum Scope<'a> {
    All,
    Pipeline(&'a PipelineId),
    Project(&'a ProjectId),
    Organization(&'a OrganizationId),
}

/// Row shape for `SELECT ... FROM jobs`. Used with `query_as!` so the JSONB
/// column and the `Option<DateTime>` columns get statically type-checked.
#[derive(sqlx::FromRow)]
struct JobRow {
    id: String,
    pipeline_id: String,
    status: String,
    node_executions: Json<Vec<JobNode>>,
    inputs: Json<Vec<(String, String)>>,
    origin: Json<JobOrigin>,
    agent_app_id: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

impl TryFrom<JobRow> for Job {
    type Error = DomainError;
    fn try_from(r: JobRow) -> DomainResult<Self> {
        let status = JobStatus::new(r.status).db_field("job status")?;
        Ok(Job::from_persistence(
            JobId::new(r.id),
            PipelineId::new(r.pipeline_id),
            status,
            r.node_executions.0,
            r.inputs.0,
            r.origin.0,
            r.agent_app_id.map(AppId::new),
            r.created_at,
            r.updated_at,
            r.started_at,
            r.finished_at,
        ))
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    pub async fn create<'e, E>(executor: E, job: &Job) -> DomainResult<Job>
    where
        E: PgExecutor<'e>,
    {
        let nodes = Json(job.node_executions().to_vec());
        let inputs = Json(job.inputs().to_vec());
        let origin = Json(job.origin().clone());
        sqlx::query!(
            r#"
            INSERT INTO jobs (id, pipeline_id, status, node_executions, inputs, origin, agent_app_id, created_at, updated_at, started_at, finished_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            job.id().as_str(),
            job.pipeline_id().as_str(),
            job.status().as_str(),
            nodes as _,
            inputs as _,
            origin as _,
            job.agent_app_id().map(AppId::as_str),
            job.created_at(),
            job.updated_at(),
            job.started_at(),
            job.finished_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(job.clone())
    }

    /// Targeted attribution write: set only `agent_app_id` so it can't clobber
    /// concurrent status / node_executions updates from the agent stream.
    pub async fn set_agent<'e, E>(executor: E, job_id: &JobId, app_id: &AppId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            "UPDATE jobs SET agent_app_id = $2 WHERE id = $1",
            job_id.as_str(),
            app_id.as_str(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    /// Pending jobs with no agent yet — the backlog to (re)dispatch when a
    /// worker connects. Oldest first so the queue drains FIFO.
    pub async fn list_pending_unassigned<'e, E>(executor: E) -> DomainResult<Vec<Job>>
    where
        E: PgExecutor<'e>,
    {
        let rows: Vec<JobRow> = sqlx::query_as!(
            JobRow,
            r#"
            SELECT id, pipeline_id, status,
                   node_executions AS "node_executions: Json<Vec<JobNode>>",
                   inputs AS "inputs: Json<Vec<(String, String)>>",
                   origin AS "origin: Json<JobOrigin>",
                   agent_app_id,
                   created_at, updated_at, started_at, finished_at
            FROM jobs
            WHERE status = $1 AND agent_app_id IS NULL
            ORDER BY created_at ASC
            "#,
            JobStatus::Pending.as_str(),
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter().map(Job::try_from).collect()
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &JobId) -> DomainResult<Job>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query_as!(
            JobRow,
            r#"
            SELECT id, pipeline_id, status,
                   node_executions AS "node_executions: Json<Vec<JobNode>>",
                   inputs AS "inputs: Json<Vec<(String, String)>>",
                   origin AS "origin: Json<JobOrigin>",
                   agent_app_id,
                   created_at, updated_at, started_at, finished_at
            FROM jobs
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("Job", id.to_string())?
        .try_into()
    }

    pub async fn update<'e, E>(executor: E, job: &Job) -> DomainResult<Job>
    where
        E: PgExecutor<'e>,
    {
        let nodes = Json(job.node_executions().to_vec());
        let res = sqlx::query!(
            r#"
            UPDATE jobs
            SET pipeline_id = $2,
                status = $3,
                node_executions = $4,
                updated_at = $5,
                started_at = $6,
                finished_at = $7
            WHERE id = $1
            "#,
            job.id().as_str(),
            job.pipeline_id().as_str(),
            job.status().as_str(),
            nodes as _,
            job.updated_at(),
            job.started_at(),
            job.finished_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        if res.rows_affected() == 0 {
            return Err(DomainError::not_found("Job", job.id().to_string()));
        }
        Ok(job.clone())
    }

    pub async fn delete<'e, E>(executor: E, id: &JobId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!("DELETE FROM jobs WHERE id = $1", id.as_str())
            .execute(executor)
            .await
            .to_domain()?;
        Ok(())
    }

    pub async fn count<'e, E>(executor: E, scope: Scope<'_>) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let count: i64 = match scope {
            Scope::All => sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM jobs"#)
                .fetch_one(executor)
                .await
                .to_domain()?,
            Scope::Pipeline(pipeline_id) => sqlx::query_scalar!(
                r#"SELECT COUNT(*) AS "count!" FROM jobs WHERE pipeline_id = $1"#,
                pipeline_id.as_str(),
            )
            .fetch_one(executor)
            .await
            .to_domain()?,
            Scope::Project(project_id) => sqlx::query_scalar!(
                r#"
                SELECT COUNT(*) AS "count!"
                FROM jobs j
                JOIN pipelines p ON p.id = j.pipeline_id
                WHERE p.project_id = $1
                "#,
                project_id.as_str(),
            )
            .fetch_one(executor)
            .await
            .to_domain()?,
            Scope::Organization(org_id) => sqlx::query_scalar!(
                r#"
                SELECT COUNT(*) AS "count!"
                FROM jobs j
                JOIN pipelines p ON p.id = j.pipeline_id
                JOIN projects pr ON pr.id = p.project_id
                WHERE pr.organization_id = $1
                "#,
                org_id.as_str(),
            )
            .fetch_one(executor)
            .await
            .to_domain()?,
        };
        Ok(u64::try_from(count).unwrap_or(0))
    }

    #[allow(clippy::too_many_lines)] // four near-identical query_as! branches
    pub async fn list_page<'e, E>(
        executor: E,
        params: &PaginationParams,
        scope: Scope<'_>,
    ) -> DomainResult<Vec<Job>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows: Vec<JobRow> = match scope {
            Scope::All => sqlx::query_as!(
                JobRow,
                r#"
                SELECT id, pipeline_id, status,
                       node_executions AS "node_executions: Json<Vec<JobNode>>",
                       inputs AS "inputs: Json<Vec<(String, String)>>",
                       origin AS "origin: Json<JobOrigin>",
                       agent_app_id,
                       created_at, updated_at, started_at, finished_at
                FROM jobs
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset,
            )
            .fetch_all(executor)
            .await
            .to_domain()?,
            Scope::Pipeline(pipeline_id) => sqlx::query_as!(
                JobRow,
                r#"
                SELECT id, pipeline_id, status,
                       node_executions AS "node_executions: Json<Vec<JobNode>>",
                       inputs AS "inputs: Json<Vec<(String, String)>>",
                       origin AS "origin: Json<JobOrigin>",
                       agent_app_id,
                       created_at, updated_at, started_at, finished_at
                FROM jobs
                WHERE pipeline_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                pipeline_id.as_str(),
                limit,
                offset,
            )
            .fetch_all(executor)
            .await
            .to_domain()?,
            Scope::Project(project_id) => sqlx::query_as!(
                JobRow,
                r#"
                SELECT j.id, j.pipeline_id, j.status,
                       j.node_executions AS "node_executions: Json<Vec<JobNode>>",
                       j.inputs AS "inputs: Json<Vec<(String, String)>>",
                       j.origin AS "origin: Json<JobOrigin>",
                       j.agent_app_id,
                       j.created_at, j.updated_at, j.started_at, j.finished_at
                FROM jobs j
                JOIN pipelines p ON p.id = j.pipeline_id
                WHERE p.project_id = $1
                ORDER BY j.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                project_id.as_str(),
                limit,
                offset,
            )
            .fetch_all(executor)
            .await
            .to_domain()?,
            Scope::Organization(org_id) => sqlx::query_as!(
                JobRow,
                r#"
                SELECT j.id, j.pipeline_id, j.status,
                       j.node_executions AS "node_executions: Json<Vec<JobNode>>",
                       j.inputs AS "inputs: Json<Vec<(String, String)>>",
                       j.origin AS "origin: Json<JobOrigin>",
                       j.agent_app_id,
                       j.created_at, j.updated_at, j.started_at, j.finished_at
                FROM jobs j
                JOIN pipelines p ON p.id = j.pipeline_id
                JOIN projects pr ON pr.id = p.project_id
                WHERE pr.organization_id = $1
                ORDER BY j.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                org_id.as_str(),
                limit,
                offset,
            )
            .fetch_all(executor)
            .await
            .to_domain()?,
        };
        rows.into_iter().map(Job::try_from).collect()
    }
}
