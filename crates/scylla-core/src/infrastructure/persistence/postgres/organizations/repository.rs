use crate::application::OrganizationRepository;
use crate::application::permission::grant::Grant;
use crate::domain::entities::{Organization, OrganizationId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};
use super::super::{grants, user_organization};

#[derive(Clone)]
pub struct PgOrganizationRepository {
    pool: PgPool,
}

impl PgOrganizationRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationRepository for PgOrganizationRepository {
    #[instrument(skip(self, organization), fields(org_id = %organization.id()))]
    async fn create(&self, organization: &Organization) -> DomainResult<Organization> {
        queries::create(&self.pool, organization).await
    }

    #[instrument(skip(self, organization, grant), fields(org_id = %organization.id(), owner = %owner))]
    async fn provision_with_owner(
        &self,
        organization: &Organization,
        owner: &UserId,
        grant: &Grant,
    ) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;
        queries::create(&mut *tx, organization).await?;
        user_organization::repository::queries::add_member(&mut *tx, owner, organization.id())
            .await?;
        grants::insert(&mut *tx, grant).await?;
        tx.commit().await.to_domain()?;
        Ok(())
    }

    #[instrument(skip(self), fields(org_id = %id))]
    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip(self), fields(name = %name))]
    async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization> {
        queries::find_by_name(&self.pool, name).await
    }

    #[instrument(skip(self, organization), fields(org_id = %organization.id()))]
    async fn update(&self, organization: &Organization) -> DomainResult<Organization> {
        queries::update(&self.pool, organization).await
    }

    #[instrument(skip(self), fields(org_id = %id))]
    async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
        queries::delete(&self.pool, id).await
    }

    #[instrument(skip(self))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_all(&self.pool).await?;
        let items = queries::list_page(&self.pool, &params, false).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self))]
    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_active(&self.pool).await?;
        let items = queries::list_page(&self.pool, &params, true).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(name = %name))]
    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool> {
        queries::name_exists(&self.pool, name).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    fn row_into_org(
        id: String,
        name: String,
        description: Option<String>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Organization> {
        let name = OrganizationName::new(name).db_field("org name")?;
        let description = description
            .map(OrganizationDescription::new)
            .transpose()
            .db_field("org description")?;
        Ok(Organization::from_persistence(
            OrganizationId::new(id),
            name,
            description,
            is_active,
            created_at,
            updated_at,
        ))
    }

    pub async fn create<'e, E>(executor: E, org: &Organization) -> DomainResult<Organization>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO organizations (id, name, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            org.id().as_str(),
            org.name().as_str(),
            org.description().map(OrganizationDescription::as_str),
            org.is_active(),
            org.created_at(),
            org.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(org.clone())
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &OrganizationId) -> DomainResult<Organization>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, name, description, is_active, created_at, updated_at
            FROM organizations
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("Organization", id.to_string())?;
        row_into_org(
            rec.id,
            rec.name,
            rec.description,
            rec.is_active,
            rec.created_at,
            rec.updated_at,
        )
    }

    pub async fn find_by_name<'e, E>(
        executor: E,
        name: &OrganizationName,
    ) -> DomainResult<Organization>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, name, description, is_active, created_at, updated_at
            FROM organizations
            WHERE name = $1
            LIMIT 1
            "#,
            name.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("Organization", name.as_str().to_string())?;
        row_into_org(
            rec.id,
            rec.name,
            rec.description,
            rec.is_active,
            rec.created_at,
            rec.updated_at,
        )
    }

    pub async fn update<'e, E>(executor: E, org: &Organization) -> DomainResult<Organization>
    where
        E: PgExecutor<'e>,
    {
        let res = sqlx::query!(
            r#"
            UPDATE organizations
            SET name = $2,
                description = $3,
                is_active = $4,
                updated_at = $5
            WHERE id = $1
            "#,
            org.id().as_str(),
            org.name().as_str(),
            org.description().map(OrganizationDescription::as_str),
            org.is_active(),
            org.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        if res.rows_affected() == 0 {
            return Err(DomainError::not_found("Organization", org.id().to_string()));
        }
        Ok(org.clone())
    }

    pub async fn delete<'e, E>(executor: E, id: &OrganizationId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!("DELETE FROM organizations WHERE id = $1", id.as_str())
            .execute(executor)
            .await
            .to_domain()?;
        Ok(())
    }

    pub async fn count_all<'e, E>(executor: E) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(r#"SELECT COUNT(*) AS "count!" FROM organizations"#)
            .fetch_one(executor)
            .await
            .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn count_active<'e, E>(executor: E) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(r#"SELECT COUNT(*) AS "count!" FROM organizations WHERE is_active"#)
            .fetch_one(executor)
            .await
            .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_page<'e, E>(
        executor: E,
        params: &PaginationParams,
        only_active: bool,
    ) -> DomainResult<Vec<Organization>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows = sqlx::query!(
            r#"
            SELECT id, name, description, is_active, created_at, updated_at
            FROM organizations
            WHERE NOT $3 OR is_active
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
            only_active,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter()
            .map(|r| {
                row_into_org(
                    r.id,
                    r.name,
                    r.description,
                    r.is_active,
                    r.created_at,
                    r.updated_at,
                )
            })
            .collect()
    }

    pub async fn name_exists<'e, E>(executor: E, name: &OrganizationName) -> DomainResult<bool>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM organizations WHERE name = $1) AS "exists!""#,
            name.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(row.exists)
    }
}
