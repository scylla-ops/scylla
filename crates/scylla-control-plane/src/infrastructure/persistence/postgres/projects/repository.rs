use crate::application::ProjectRepository;
use crate::application::authz::Visibility;
use crate::application::authz::grant::Grant;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{OrganizationId, ProjectId, UserId};
use crate::domain::project::Project;
use crate::domain::project::{ProjectDescription, ProjectName};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};
use super::super::grants;

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
    #[instrument(skip_all, fields(project_id = %project.id()))]
    async fn create(&self, project: &Project) -> DomainResult<Project> {
        queries::create(&self.pool, project).await
    }

    #[instrument(skip_all, fields(project_id = %project.id()))]
    async fn provision_with_owner(&self, project: &Project, grant: &Grant) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;
        queries::create(&mut *tx, project).await?;
        grants::insert(&mut *tx, grant).await?;
        tx.commit().await.to_domain()?;
        Ok(())
    }

    #[instrument(skip_all, fields(project_id = %project_id))]
    async fn list_principals(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_principals(&self.pool, project_id).await?;
        let items = queries::list_principals_page(&self.pool, project_id, &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip_all, fields(user_id = %user_id))]
    async fn list_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_for_user(&self.pool, user_id).await?;
        let items = queries::list_for_user_page(&self.pool, user_id, &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip_all, fields(project_id = %id))]
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip_all, fields(n = ids.len()))]
    async fn find_by_ids(&self, ids: &[ProjectId]) -> DomainResult<Vec<Project>> {
        queries::find_by_ids(&self.pool, ids).await
    }

    #[instrument(skip_all, fields(project_id = %project.id()))]
    async fn update(&self, project: &Project) -> DomainResult<Project> {
        queries::update(&self.pool, project).await
    }

    #[instrument(skip_all, fields(project_id = %id))]
    async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        queries::delete(&self.pool, id).await
    }

    #[instrument(skip(self, pagination))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let params = pagination.copied().unwrap_or_default();
        let unrestricted = queries::VisibilityFilter::unrestricted();
        let total = queries::count(&self.pool, false, None, &unrestricted).await?;
        let items = queries::list_page(&self.pool, &params, false, None, &unrestricted).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self, pagination))]
    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let params = pagination.copied().unwrap_or_default();
        let unrestricted = queries::VisibilityFilter::unrestricted();
        let total = queries::count(&self.pool, true, None, &unrestricted).await?;
        let items = queries::list_page(&self.pool, &params, true, None, &unrestricted).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip_all, fields(org_id = %organization_id))]
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
        visible: &Visibility,
    ) -> DomainResult<PaginatedResult<Project>> {
        let params = pagination.copied().unwrap_or_default();
        // Nothing visible: answer an empty page without touching the database.
        if visible.is_empty() {
            return Ok(PaginatedResult::new(Vec::new(), &params, 0));
        }
        let filter = queries::VisibilityFilter::new(visible);
        let total = queries::count(&self.pool, false, Some(organization_id), &filter).await?;
        let items =
            queries::list_page(&self.pool, &params, false, Some(organization_id), &filter).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip_all, fields(org_id = %organization_id))]
    async fn count_by_organization(&self, organization_id: &OrganizationId) -> DomainResult<u64> {
        queries::count(
            &self.pool,
            false,
            Some(organization_id),
            &queries::VisibilityFilter::unrestricted(),
        )
        .await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    pub async fn count_principals<'e, E>(executor: E, project_id: &ProjectId) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT principal_id) AS "count!" FROM grants
            WHERE principal_kind = 'user' AND scope_kind = 'project' AND scope_id = $1
            "#,
            project_id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_principals_page<'e, E>(
        executor: E,
        project_id: &ProjectId,
        params: &PaginationParams,
    ) -> DomainResult<Vec<UserId>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows = sqlx::query!(
            r#"
            SELECT principal_id, MAX(created_at) AS "granted_at!" FROM grants
            WHERE principal_kind = 'user' AND scope_kind = 'project' AND scope_id = $1
            GROUP BY principal_id
            ORDER BY "granted_at!" DESC
            LIMIT $2 OFFSET $3
            "#,
            project_id.as_str(),
            limit,
            offset,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        Ok(rows
            .into_iter()
            .map(|r| UserId::new(r.principal_id))
            .collect())
    }

    /// Projects reachable by a user: granted directly, or through a grant on the
    /// owning organization.
    ///
    /// ponytail: the org arm re-reads `grants` per call; fold it into the
    /// `Visibility` resolver if project listings ever get hot.
    pub async fn count_for_user<'e, E>(executor: E, user_id: &UserId) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!" FROM projects p
            WHERE p.id IN (
                SELECT scope_id FROM grants
                WHERE principal_kind = 'user' AND principal_id = $1 AND scope_kind = 'project'
              )
              OR p.organization_id IN (
                SELECT scope_id FROM grants
                WHERE principal_kind = 'user' AND principal_id = $1 AND scope_kind = 'organization'
              )
            "#,
            user_id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_for_user_page<'e, E>(
        executor: E,
        user_id: &UserId,
        params: &PaginationParams,
    ) -> DomainResult<Vec<Project>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows = sqlx::query!(
            r#"
            SELECT id, name, description, organization_id, is_active, created_at, updated_at
            FROM projects p
            WHERE p.id IN (
                SELECT scope_id FROM grants
                WHERE principal_kind = 'user' AND principal_id = $1 AND scope_kind = 'project'
              )
              OR p.organization_id IN (
                SELECT scope_id FROM grants
                WHERE principal_kind = 'user' AND principal_id = $1 AND scope_kind = 'organization'
              )
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id.as_str(),
            limit,
            offset,
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

    pub async fn find_by_ids<'e, E>(executor: E, ids: &[ProjectId]) -> DomainResult<Vec<Project>>
    where
        E: PgExecutor<'e>,
    {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let id_strs: Vec<String> = ids.iter().map(|i| i.as_str().to_owned()).collect();
        let rows = sqlx::query!(
            r#"
            SELECT id, name, description, organization_id, is_active, created_at, updated_at
            FROM projects
            WHERE id = ANY($1::text[])
            "#,
            &id_strs,
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

    /// A [`Visibility`] flattened into the three bind parameters the queries
    /// below take: see-everything, plus the organizations and projects the
    /// caller holds. Built once per call so the count and the page share exactly
    /// the same filter.
    pub struct VisibilityFilter {
        all: bool,
        orgs: Vec<String>,
        projects: Vec<String>,
    }

    impl VisibilityFilter {
        #[must_use]
        pub fn new(visible: &Visibility) -> Self {
            match visible {
                Visibility::All => Self {
                    all: true,
                    orgs: Vec::new(),
                    projects: Vec::new(),
                },
                Visibility::Scoped { orgs, projects } => Self {
                    all: false,
                    orgs: orgs.iter().map(|o| o.as_str().to_owned()).collect(),
                    projects: projects.iter().map(|p| p.as_str().to_owned()).collect(),
                },
            }
        }

        /// Everything is visible — used by the internal callers that legitimately
        /// bypass filtering (quota counts, cascade bookkeeping).
        #[must_use]
        pub fn unrestricted() -> Self {
            Self {
                all: true,
                orgs: Vec::new(),
                projects: Vec::new(),
            }
        }
    }

    pub async fn count<'e, E>(
        executor: E,
        only_active: bool,
        organization_id: Option<&OrganizationId>,
        visible: &VisibilityFilter,
    ) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!" FROM projects
            WHERE (NOT $1 OR is_active)
              AND ($2::text IS NULL OR organization_id = $2)
              AND ($3 OR organization_id = ANY($4) OR id = ANY($5))
            "#,
            only_active,
            organization_id.map(AsRef::as_ref),
            visible.all,
            &visible.orgs,
            &visible.projects,
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
        visible: &VisibilityFilter,
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
              AND ($5 OR organization_id = ANY($6) OR id = ANY($7))
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
            only_active,
            organization_id.map(AsRef::as_ref),
            visible.all,
            &visible.orgs,
            &visible.projects,
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
