use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{OrganizationId, ProjectId, UserId};
use crate::domain::project::Project;
use crate::domain::project::{ProjectDescription, ProjectName};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla_auth::authz::{Grant, Visibility};
use scylla_core::application::ProjectRepository;
use scylla_core::application::pagination::{PaginatedResult, PaginationParams};
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

    // A write that matched no row is a stale read if the row still exists, else a missing row.
    async fn stale(&self, project: &Project) -> DomainError {
        match queries::exists(&self.pool, project.id()).await {
            Ok(true) => DomainError::conflict(format!(
                "project {} changed since it was read",
                project.id()
            )),
            Ok(false) => DomainError::not_found("Project", project.id().to_string()),
            Err(e) => e,
        }
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

    #[instrument(skip_all, fields(project_id = %project.id(), version = project.version()))]
    async fn update(&self, project: &Project) -> DomainResult<Project> {
        match queries::update(&self.pool, project).await? {
            Some(updated) => Ok(updated),
            None => Err(self.stale(project).await),
        }
    }

    #[instrument(skip_all, fields(project_id = %project.id(), version = project.version()))]
    async fn delete(&self, project: &Project) -> DomainResult<()> {
        if queries::delete(&self.pool, project).await? {
            Ok(())
        } else {
            Err(self.stale(project).await)
        }
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

    #[instrument(skip_all, fields(org_id = %organization_id))]
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
        visible: &Visibility,
    ) -> DomainResult<PaginatedResult<Project>> {
        let params = pagination.copied().unwrap_or_default();
        if visible.is_empty() {
            return Ok(PaginatedResult::new(Vec::new(), &params, 0));
        }
        let filter = queries::VisibilityFilter::new(visible);
        let total = queries::count(&self.pool, false, Some(organization_id), &filter).await?;
        let items =
            queries::list_page(&self.pool, &params, false, Some(organization_id), &filter).await?;
        Ok(PaginatedResult::new(items, &params, total))
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

    // ponytail: the org arm re-reads `grants` per call; fold into `Visibility` if listings get hot.
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
            SELECT id, name, description, organization_id, is_active, created_at, updated_at, version
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
                    r.version,
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
        version: i64,
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
            u64::try_from(version).unwrap_or(0),
        ))
    }

    pub async fn create<'e, E>(executor: E, project: &Project) -> DomainResult<Project>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO projects (id, name, description, organization_id, is_active, created_at, updated_at, version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            project.id().as_str(),
            project.name().as_str(),
            project.description().map(ProjectDescription::as_str),
            project.organization_id().as_str(),
            project.is_active(),
            project.created_at(),
            project.updated_at(),
            version_column(project),
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
            SELECT id, name, description, organization_id, is_active, created_at, updated_at, version
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
            rec.version,
        )
    }

    /// `None` when no row carries the staged version: the caller tells a stale read from a
    /// missing row with `exists`, on its own statement.
    pub async fn update<'e, E>(executor: E, project: &Project) -> DomainResult<Option<Project>>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            UPDATE projects
            SET name = $2,
                description = $3,
                organization_id = $4,
                is_active = $5,
                updated_at = $6,
                version = version + 1
            WHERE id = $1 AND version = $7
            RETURNING id, name, description, organization_id, is_active, created_at, updated_at, version
            "#,
            project.id().as_str(),
            project.name().as_str(),
            project.description().map(ProjectDescription::as_str),
            project.organization_id().as_str(),
            project.is_active(),
            project.updated_at(),
            version_column(project),
        )
        .fetch_optional(executor)
        .await
        .to_domain()?;
        rec.map(|r| {
            row_into_project(
                r.id,
                r.name,
                r.description,
                r.organization_id,
                r.is_active,
                r.created_at,
                r.updated_at,
                r.version,
            )
        })
        .transpose()
    }

    /// `false` when no row carries the staged version.
    pub async fn delete<'e, E>(executor: E, project: &Project) -> DomainResult<bool>
    where
        E: PgExecutor<'e>,
    {
        let res = sqlx::query!(
            "DELETE FROM projects WHERE id = $1 AND version = $2",
            project.id().as_str(),
            version_column(project),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn exists<'e, E>(executor: E, id: &ProjectId) -> DomainResult<bool>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1) AS "exists!""#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(rec.exists)
    }

    fn version_column(project: &Project) -> i64 {
        i64::try_from(project.version()).unwrap_or(i64::MAX)
    }

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
            SELECT id, name, description, organization_id, is_active, created_at, updated_at, version
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
                    r.version,
                )
            })
            .collect()
    }
}
