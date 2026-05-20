use crate::application::ProjectRepository;
use crate::domain::entities::{OrganizationId, Project, ProjectId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::project::{ProjectDescription, ProjectName};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};

#[derive(Clone)]
pub struct PgProjectRepository {
    pool: PgPool,
}

impl PgProjectRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    #[instrument(skip(self, project), fields(project_id = %project.id()))]
    async fn create(&self, project: &Project) -> DomainResult<Project> {
        queries::create(&self.pool, project).await
    }

    #[instrument(skip(self), fields(project_id = %id))]
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip(self, project), fields(project_id = %project.id()))]
    async fn update(&self, project: &Project) -> DomainResult<Project> {
        queries::update(&self.pool, project).await
    }

    #[instrument(skip(self), fields(project_id = %id))]
    async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        queries::delete(&self.pool, id).await
    }

    #[instrument(skip(self))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count(&self.pool, false, None).await?;
        let items = queries::list_page(&self.pool, &params, false, None).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self))]
    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count(&self.pool, true, None).await?;
        let items = queries::list_page(&self.pool, &params, true, None).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(org_id = %organization_id))]
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count(&self.pool, false, Some(organization_id)).await?;
        let items = queries::list_page(&self.pool, &params, false, Some(organization_id)).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(org_id = %organization_id))]
    async fn count_by_organization(&self, organization_id: &OrganizationId) -> DomainResult<u64> {
        queries::count(&self.pool, false, Some(organization_id)).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn row_into_project(
        id: String,
        name: String,
        description: Option<String>,
        organization_id: String,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Project> {
        let name = ProjectName::new(name).db_field("project name")?;
        let description = description
            .map(ProjectDescription::new)
            .transpose()
            .db_field("project description")?;
        Ok(Project::from_persistence(
            ProjectId::new(id),
            name,
            description,
            OrganizationId::new(organization_id),
            is_active,
            created_at,
            updated_at,
        ))
    }

    pub async fn create<'e, E>(executor: E, project: &Project) -> DomainResult<Project>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO projects (id, name, description, organization_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            project.id().as_str(),
            project.name().as_str(),
            project.description().map(ProjectDescription::as_str),
            project.organization_id().as_str(),
            project.is_active(),
            project.created_at(),
            project.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(project.clone())
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &ProjectId) -> DomainResult<Project>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, name, description, organization_id, is_active, created_at, updated_at
            FROM projects
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("Project", id.to_string())?;
        row_into_project(
            rec.id,
            rec.name,
            rec.description,
            rec.organization_id,
            rec.is_active,
            rec.created_at,
            rec.updated_at,
        )
    }

    pub async fn update<'e, E>(executor: E, project: &Project) -> DomainResult<Project>
    where
        E: PgExecutor<'e>,
    {
        let res = sqlx::query!(
            r#"
            UPDATE projects
            SET name = $2,
                description = $3,
                organization_id = $4,
                is_active = $5,
                updated_at = $6
            WHERE id = $1
            "#,
            project.id().as_str(),
            project.name().as_str(),
            project.description().map(ProjectDescription::as_str),
            project.organization_id().as_str(),
            project.is_active(),
            project.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        if res.rows_affected() == 0 {
            return Err(DomainError::not_found("Project", project.id().to_string()));
        }
        Ok(project.clone())
    }

    pub async fn delete<'e, E>(executor: E, id: &ProjectId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!("DELETE FROM projects WHERE id = $1", id.as_str())
            .execute(executor)
            .await
            .to_domain()?;
        Ok(())
    }

    pub async fn count<'e, E>(
        executor: E,
        only_active: bool,
        organization_id: Option<&OrganizationId>,
    ) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!" FROM projects
            WHERE (NOT $1 OR is_active)
              AND ($2::text IS NULL OR organization_id = $2)
            "#,
            only_active,
            organization_id.map(AsRef::as_ref),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_page<'e, E>(
        executor: E,
        params: &PaginationParams,
        only_active: bool,
        organization_id: Option<&OrganizationId>,
    ) -> DomainResult<Vec<Project>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows = sqlx::query!(
            r#"
            SELECT id, name, description, organization_id, is_active, created_at, updated_at
            FROM projects
            WHERE (NOT $3 OR is_active)
              AND ($4::text IS NULL OR organization_id = $4)
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
            only_active,
            organization_id.map(AsRef::as_ref),
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter()
            .map(|r| {
                row_into_project(
                    r.id,
                    r.name,
                    r.description,
                    r.organization_id,
                    r.is_active,
                    r.created_at,
                    r.updated_at,
                )
            })
            .collect()
    }
}
