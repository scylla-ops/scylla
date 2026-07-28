use crate::application::app::AppRepository;
use crate::application::authz::grant::Grant;
use crate::domain::agent::Agent;
use crate::domain::app::AppName;
use crate::domain::app::{App, AppCredential};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, OrganizationId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};
use super::super::{agents, app_secrets, grants};

#[derive(Clone)]
pub struct PgAppRepository {
    pool: PgPool,
}

impl PgAppRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AppRepository for PgAppRepository {
    #[instrument(skip(self, app, credential), fields(app_id = %app.id(), org_id = %app.organization_id()))]
    async fn create_app(&self, app: &App, credential: &AppCredential) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;
        queries::create(&mut *tx, app).await?;
        app_secrets::insert(&mut *tx, credential).await?;
        tx.commit().await.to_domain()?;
        Ok(())
    }

    #[instrument(skip(self, app, credential, agent, grant), fields(app_id = %app.id(), org_id = %app.organization_id()))]
    async fn provision_agent(
        &self,
        app: &App,
        credential: &AppCredential,
        agent: &Agent,
        grant: &Grant,
    ) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;
        queries::create(&mut *tx, app).await?;
        app_secrets::insert(&mut *tx, credential).await?;
        agents::insert(&mut *tx, agent).await?;
        grants::insert(&mut *tx, grant).await?;
        tx.commit().await.to_domain()?;
        Ok(())
    }

    #[instrument(skip(self, app, credential, grant), fields(app_id = %app.id(), org_id = %app.organization_id()))]
    async fn provision(
        &self,
        app: &App,
        credential: &AppCredential,
        grant: &Grant,
    ) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;
        queries::create(&mut *tx, app).await?;
        app_secrets::insert(&mut *tx, credential).await?;
        grants::insert(&mut *tx, grant).await?;
        tx.commit().await.to_domain()?;
        Ok(())
    }

    #[instrument(skip(self), fields(app_id = %id))]
    async fn find_by_id(&self, id: &AppId) -> DomainResult<App> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip(self), fields(org_id = %organization_id))]
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
    ) -> DomainResult<Vec<App>> {
        queries::list_by_organization(&self.pool, organization_id).await
    }

    #[instrument(skip(self), fields(app_id = %id, active))]
    async fn set_active(&self, id: &AppId, active: bool) -> DomainResult<()> {
        queries::set_active(&self.pool, id, active).await
    }

    #[instrument(skip(self), fields(app_id = %id))]
    async fn delete(&self, id: &AppId) -> DomainResult<()> {
        queries::delete(&self.pool, id).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    fn row_into_app(
        id: String,
        organization_id: String,
        name: String,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<App> {
        let name = AppName::new(name).db_field("app name")?;
        Ok(App::from_persistence(
            AppId::new(id),
            OrganizationId::new(organization_id),
            name,
            is_active,
            created_at,
            updated_at,
        ))
    }

    pub async fn create<'e, E>(executor: E, app: &App) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO apps (id, organization_id, name, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            app.id().as_str(),
            app.organization_id().as_str(),
            app.name().as_str(),
            app.is_active(),
            app.created_at(),
            app.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &AppId) -> DomainResult<App>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, organization_id, name, is_active, created_at, updated_at
            FROM apps
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("App", id.to_string())?;
        row_into_app(
            rec.id,
            rec.organization_id,
            rec.name,
            rec.is_active,
            rec.created_at,
            rec.updated_at,
        )
    }

    pub async fn list_by_organization<'e, E>(
        executor: E,
        organization_id: &OrganizationId,
    ) -> DomainResult<Vec<App>>
    where
        E: PgExecutor<'e>,
    {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, name, is_active, created_at, updated_at
            FROM apps
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
            organization_id.as_str(),
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter()
            .map(|r| {
                row_into_app(
                    r.id,
                    r.organization_id,
                    r.name,
                    r.is_active,
                    r.created_at,
                    r.updated_at,
                )
            })
            .collect()
    }

    pub async fn set_active<'e, E>(executor: E, id: &AppId, active: bool) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            "UPDATE apps SET is_active = $2, updated_at = NOW() WHERE id = $1",
            id.as_str(),
            active,
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(())
    }

    pub async fn delete<'e, E>(executor: E, id: &AppId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!("DELETE FROM apps WHERE id = $1", id.as_str())
            .execute(executor)
            .await
            .to_domain()?;
        Ok(())
    }
}
