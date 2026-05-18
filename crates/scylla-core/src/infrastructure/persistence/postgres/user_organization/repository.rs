use crate::application::ports::UserOrganizationRepository;
use crate::domain::entities::{OrganizationId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::SqlxResultExt;

#[derive(Clone)]
pub struct PgUserOrganizationRepository {
    pool: PgPool,
}

impl PgUserOrganizationRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserOrganizationRepository for PgUserOrganizationRepository {
    #[instrument(skip(self))]
    async fn add_member(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<()> {
        queries::add_member(&self.pool, user_id, org_id).await
    }

    #[instrument(skip(self))]
    async fn remove_member(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<()> {
        queries::remove_member(&self.pool, user_id, org_id).await
    }

    #[instrument(skip(self))]
    async fn is_member(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<bool> {
        queries::is_member(&self.pool, user_id, org_id).await
    }

    #[instrument(skip(self))]
    async fn list_members(
        &self,
        org_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_members(&self.pool, org_id).await?;
        let items = queries::list_members(&self.pool, org_id, &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self))]
    async fn list_user_organizations(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<OrganizationId>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_user_organizations(&self.pool, user_id).await?;
        let items = queries::list_user_organizations(&self.pool, user_id, &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    pub async fn add_member<'e, E>(
        executor: E,
        user_id: &UserId,
        org_id: &OrganizationId,
    ) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO user_organization (user_id, organization_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            user_id.as_str(),
            org_id.as_str(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    pub async fn remove_member<'e, E>(
        executor: E,
        user_id: &UserId,
        org_id: &OrganizationId,
    ) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            "DELETE FROM user_organization WHERE user_id = $1 AND organization_id = $2",
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
        org_id: &OrganizationId,
    ) -> DomainResult<bool>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_organization WHERE user_id = $1 AND organization_id = $2
            ) AS "exists!"
            "#,
            user_id.as_str(),
            org_id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(row.exists)
    }

    pub async fn count_members<'e, E>(executor: E, org_id: &OrganizationId) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) AS "count!" FROM user_organization WHERE organization_id = $1"#,
            org_id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_members<'e, E>(
        executor: E,
        org_id: &OrganizationId,
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
            FROM user_organization
            WHERE organization_id = $1
            ORDER BY user_id
            LIMIT $2 OFFSET $3
            "#,
            org_id.as_str(),
            limit,
            offset,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        Ok(rows.into_iter().map(|r| UserId::new(r.user_id)).collect())
    }

    pub async fn count_user_organizations<'e, E>(executor: E, user_id: &UserId) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) AS "count!" FROM user_organization WHERE user_id = $1"#,
            user_id.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_user_organizations<'e, E>(
        executor: E,
        user_id: &UserId,
        params: &PaginationParams,
    ) -> DomainResult<Vec<OrganizationId>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows = sqlx::query!(
            r#"
            SELECT organization_id
            FROM user_organization
            WHERE user_id = $1
            ORDER BY organization_id
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
            .map(|r| OrganizationId::new(r.organization_id))
            .collect())
    }
}
