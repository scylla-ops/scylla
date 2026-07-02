use crate::application::UserProjectRepository;
use crate::domain::entities::{OrganizationId, ProjectId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::SqlxResultExt;

#[derive(Clone)]
pub struct PgUserProjectRepository {
    pool: PgPool,
}

impl PgUserProjectRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserProjectRepository for PgUserProjectRepository {
    #[instrument(skip(self))]
    async fn add_member(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<()> {
        queries::add_member(&self.pool, user_id, project_id).await
    }

    #[instrument(skip(self))]
    async fn remove_member(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<()> {
        queries::remove_member(&self.pool, user_id, project_id).await
    }

    #[instrument(skip(self))]
    async fn is_member(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<bool> {
        queries::is_member(&self.pool, user_id, project_id).await
    }

    #[instrument(skip(self))]
    async fn list_members(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_members(&self.pool, project_id).await?;
        let items = queries::list_members(&self.pool, project_id, &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self))]
    async fn list_user_projects(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<ProjectId>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_user_projects(&self.pool, user_id).await?;
        let items = queries::list_user_projects(&self.pool, user_id, &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    pub async fn add_member<'e, E>(
        executor: E,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO user_project (user_id, project_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            user_id.as_str(),
            project_id.as_str(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    pub async fn remove_member<'e, E>(
        executor: E,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            "DELETE FROM user_project WHERE user_id = $1 AND project_id = $2",
            user_id.as_str(),
            project_id.as_str(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    /// Remove a user from every project belonging to `org_id`, on any executor.
    /// Part of removing them from the organization: project membership makes no
    /// sense once the enclosing org membership is gone.
    pub async fn remove_member_from_org_projects<'e, E>(
        executor: E,
        user_id: &UserId,
        org_id: &OrganizationId,
    ) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            "DELETE FROM user_project \
             WHERE user_id = $1 \
               AND project_id IN (SELECT id FROM projects WHERE organization_id = $2)",
            user_id.as_str(),
            org_id.as_str(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    pub async fn is_member<'e, E>(
        executor: E,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<bool>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_project WHERE user_id = $1 AND project_id = $2
            ) AS "exists!"
            "#,
            user_id.as_str(),
            project_id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(row.exists)
    }

    pub async fn count_members<'e, E>(executor: E, project_id: &ProjectId) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) AS "count!" FROM user_project WHERE project_id = $1"#,
            project_id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_members<'e, E>(
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
            SELECT user_id
            FROM user_project
            WHERE project_id = $1
            ORDER BY user_id
            LIMIT $2 OFFSET $3
            "#,
            project_id.as_str(),
            limit,
            offset,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        Ok(rows.into_iter().map(|r| UserId::new(r.user_id)).collect())
    }

    pub async fn count_user_projects<'e, E>(executor: E, user_id: &UserId) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) AS "count!" FROM user_project WHERE user_id = $1"#,
            user_id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_user_projects<'e, E>(
        executor: E,
        user_id: &UserId,
        params: &PaginationParams,
    ) -> DomainResult<Vec<ProjectId>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows = sqlx::query!(
            r#"
            SELECT project_id
            FROM user_project
            WHERE user_id = $1
            ORDER BY project_id
            LIMIT $2 OFFSET $3
            "#,
            user_id.as_str(),
            limit,
            offset,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        Ok(rows
            .into_iter()
            .map(|r| ProjectId::new(r.project_id))
            .collect())
    }
}
