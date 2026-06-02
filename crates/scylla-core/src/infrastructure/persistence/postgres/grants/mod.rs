use crate::application::authz::grant::{Grant, GrantPrincipal, GrantRepository, GrantScope};
use crate::domain::entities::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::error::SqlxResultExt;

const SCOPE_SYSTEM: &str = "system";
const SCOPE_ORGANIZATION: &str = "organization";
const SCOPE_PROJECT: &str = "project";
/// Sentinel `scope_id` for the singleton System scope (column is NOT NULL; there
/// is only one System root, so the id is constant and ignored on read).
const SYSTEM_SCOPE_ID: &str = "system";
const PRINCIPAL_USER: &str = "user";
const PRINCIPAL_APP: &str = "app";

/// Insert a grant on any executor (pool or transaction). Idempotent via the
/// `(principal_kind, principal_id, role_name, scope_kind, scope_id)` unique
/// constraint, so re-running a signup or grant call is a no-op rather than a
/// conflict. Shared by the pool-backed repo and the atomic signup transaction.
pub async fn insert<'e, E>(executor: E, grant: &Grant) -> DomainResult<()>
where
    E: PgExecutor<'e>,
{
    let (scope_kind, scope_id) = match &grant.scope {
        GrantScope::System => (SCOPE_SYSTEM, SYSTEM_SCOPE_ID),
        GrantScope::Organization(id) => (SCOPE_ORGANIZATION, id.as_str()),
        GrantScope::Project(id) => (SCOPE_PROJECT, id.as_str()),
    };
    sqlx::query!(
        "INSERT INTO grants (id, principal_kind, principal_id, role_name, scope_kind, scope_id) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (principal_kind, principal_id, role_name, scope_kind, scope_id) DO NOTHING",
        grant.id.as_str(),
        grant.principal.kind(),
        grant.principal.id(),
        grant.role.as_str(),
        scope_kind,
        scope_id,
    )
    .execute(executor)
    .await
    .to_domain()?;
    Ok(())
}

/// Persistence for explicit scoped grants (`grants` table). Read once
/// at `CedarPermissionService` construction to link template instances.
#[derive(Clone)]
pub struct PgGrantRepository {
    pool: PgPool,
}

impl PgGrantRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GrantRepository for PgGrantRepository {
    #[instrument(skip(self))]
    async fn list_all(&self) -> DomainResult<Vec<Grant>> {
        let rows = sqlx::query!(
            "SELECT id, principal_kind, principal_id, role_name, scope_kind, scope_id \
             FROM grants",
        )
        .fetch_all(&self.pool)
        .await
        .to_domain()?;

        rows.into_iter()
            .map(|r| {
                let principal = match r.principal_kind.as_str() {
                    PRINCIPAL_USER => GrantPrincipal::User(UserId::new(r.principal_id)),
                    PRINCIPAL_APP => GrantPrincipal::App(AppId::new(r.principal_id)),
                    other => {
                        return Err(DomainError::Infrastructure(format!(
                            "unknown grant principal_kind '{other}'"
                        )));
                    }
                };
                let scope = match r.scope_kind.as_str() {
                    SCOPE_SYSTEM => GrantScope::System,
                    SCOPE_ORGANIZATION => GrantScope::Organization(OrganizationId::new(r.scope_id)),
                    SCOPE_PROJECT => GrantScope::Project(ProjectId::new(r.scope_id)),
                    other => {
                        return Err(DomainError::Infrastructure(format!(
                            "unknown grant scope_kind '{other}'"
                        )));
                    }
                };
                Ok(Grant {
                    id: r.id,
                    principal,
                    role: RoleName::new(r.role_name)?,
                    scope,
                })
            })
            .collect()
    }

    #[instrument(skip(self, grant), fields(grant_id = %grant.id))]
    async fn create(&self, grant: &Grant) -> DomainResult<()> {
        insert(&self.pool, grant).await
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &str) -> DomainResult<()> {
        sqlx::query!("DELETE FROM grants WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .to_domain()?;
        Ok(())
    }
}
