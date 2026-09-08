use crate::application::PipelineRepository;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{OrganizationId, PipelineId, ProjectId};
use crate::domain::pipeline::PipelineName;
use crate::domain::pipeline::{Pipeline, PipelineNode};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool, types::Json};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};

#[derive(Clone)]
pub struct PgPipelineRepository {
    pool: PgPool,
}

impl PgPipelineRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PipelineRepository for PgPipelineRepository {
    #[instrument(skip_all, fields(pipeline_id = %pipeline.id()))]
    async fn create(&self, pipeline: &Pipeline) -> DomainResult<Pipeline> {
        queries::create(&self.pool, pipeline).await
    }

    #[instrument(skip_all, fields(pipeline_id = %id))]
    async fn find_by_id(&self, id: &PipelineId) -> DomainResult<Pipeline> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip_all, fields(pipeline_id = %pipeline.id()))]
    async fn update(&self, pipeline: &Pipeline) -> DomainResult<Pipeline> {
        queries::update(&self.pool, pipeline).await
    }

    #[instrument(skip_all, fields(pipeline_id = %id))]
    async fn delete(&self, id: &PipelineId) -> DomainResult<()> {
        queries::delete(&self.pool, id).await
    }

    #[instrument(skip(self, pagination))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count(&self.pool, Scope::All).await?;
        let items = queries::list_page(&self.pool, &params, Scope::All).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip_all, fields(project_id = %project_id))]
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count(&self.pool, Scope::Project(project_id)).await?;
        let items = queries::list_page(&self.pool, &params, Scope::Project(project_id)).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip_all, fields(org_id = %organization_id))]
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
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
    Project(&'a ProjectId),
    Organization(&'a OrganizationId),
}

/// Row shape for `SELECT ... FROM pipelines`. `query_as!` macro expects fields
/// matching column names in order and types.
#[derive(sqlx::FromRow)]
struct PipelineRow {
    id: String,
    project_id: String,
    name: String,
    nodes: Json<Vec<PipelineNode>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<PipelineRow> for Pipeline {
    type Error = DomainError;
    fn try_from(r: PipelineRow) -> DomainResult<Self> {
        let name = PipelineName::new(r.name).db_field("pipeline name")?;
        Ok(Pipeline::from_persistence(
            PipelineId::new(r.id),
            ProjectId::new(r.project_id),
            name,
            r.nodes.0,
            r.created_at,
            r.updated_at,
        ))
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    pub async fn create<'e, E>(executor: E, pipeline: &Pipeline) -> DomainResult<Pipeline>
    where
        E: PgExecutor<'e>,
    {
        let nodes = Json(pipeline.nodes().to_vec());
        sqlx::query!(
            r#"
            INSERT INTO pipelines (id, project_id, name, nodes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            pipeline.id().as_str(),
            pipeline.project_id().as_str(),
            pipeline.name().as_str(),
            nodes as _,
            pipeline.created_at(),
            pipeline.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(pipeline.clone())
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &PipelineId) -> DomainResult<Pipeline>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query_as!(
            PipelineRow,
            r#"
            SELECT id, project_id, name,
                   nodes AS "nodes: Json<Vec<PipelineNode>>",
                   created_at, updated_at
            FROM pipelines
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("Pipeline", id.to_string())?
        .try_into()
    }

    pub async fn update<'e, E>(executor: E, pipeline: &Pipeline) -> DomainResult<Pipeline>
    where
        E: PgExecutor<'e>,
    {
        let nodes = Json(pipeline.nodes().to_vec());
        let res = sqlx::query!(
            r#"
            UPDATE pipelines
            SET project_id = $2,
                name = $3,
                nodes = $4,
                updated_at = $5
            WHERE id = $1
            "#,
            pipeline.id().as_str(),
            pipeline.project_id().as_str(),
            pipeline.name().as_str(),
            nodes as _,
            pipeline.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        if res.rows_affected() == 0 {
            return Err(DomainError::not_found(
                "Pipeline",
                pipeline.id().to_string(),
            ));
        }
        Ok(pipeline.clone())
    }

    pub async fn delete<'e, E>(executor: E, id: &PipelineId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!("DELETE FROM pipelines WHERE id = $1", id.as_str())
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
            Scope::All => sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM pipelines"#)
                .fetch_one(executor)
                .await
                .to_domain()?,
            Scope::Project(project_id) => sqlx::query_scalar!(
                r#"SELECT COUNT(*) AS "count!" FROM pipelines WHERE project_id = $1"#,
                project_id.as_str(),
            )
            .fetch_one(executor)
            .await
            .to_domain()?,
            Scope::Organization(org_id) => sqlx::query_scalar!(
                r#"
                SELECT COUNT(*) AS "count!"
                FROM pipelines p
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

    pub async fn list_page<'e, E>(
        executor: E,
        params: &PaginationParams,
        scope: Scope<'_>,
    ) -> DomainResult<Vec<Pipeline>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows: Vec<PipelineRow> = match scope {
            Scope::All => sqlx::query_as!(
                PipelineRow,
                r#"
                SELECT id, project_id, name,
                       nodes AS "nodes: Json<Vec<PipelineNode>>",
                       created_at, updated_at
                FROM pipelines
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset,
            )
            .fetch_all(executor)
            .await
            .to_domain()?,
            Scope::Project(project_id) => sqlx::query_as!(
                PipelineRow,
                r#"
                SELECT id, project_id, name,
                       nodes AS "nodes: Json<Vec<PipelineNode>>",
                       created_at, updated_at
                FROM pipelines
                WHERE project_id = $1
                ORDER BY created_at DESC
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
                PipelineRow,
                r#"
                SELECT p.id, p.project_id, p.name,
                       p.nodes AS "nodes: Json<Vec<PipelineNode>>",
                       p.created_at, p.updated_at
                FROM pipelines p
                JOIN projects pr ON pr.id = p.project_id
                WHERE pr.organization_id = $1
                ORDER BY p.created_at DESC
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
        rows.into_iter().map(Pipeline::try_from).collect()
    }
}
