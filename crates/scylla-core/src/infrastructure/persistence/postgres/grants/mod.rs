use crate::application::authz::grant::{Grant, GrantRepository, GrantTarget, Principal, Scope};
use crate::domain::entities::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::role::RoleName;
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
const TARGET_ROLE: &str = "role";
const TARGET_PERMISSION: &str = "permission";

/// Insert a grant on any executor (pool or transaction). Idempotent via the
/// `(principal_kind, principal_id, target_kind, target, scope_kind, scope_id)`
/// unique constraint, so re-running a signup or grant call is a no-op rather than
/// a conflict. Shared by the pool-backed repo and the atomic signup transaction.
pub async fn insert<'e, E>(executor: E, grant: &Grant) -> DomainResult<()>
where
    E: PgExecutor<'e>,
{
    let (scope_kind, scope_id) = match &grant.scope {
        Scope::System => (SCOPE_SYSTEM, SYSTEM_SCOPE_ID),
        Scope::Organization(id) => (SCOPE_ORGANIZATION, id.as_str()),
        Scope::Project(id) => (SCOPE_PROJECT, id.as_str()),
    };
    sqlx::query!(
        "INSERT INTO grants \
             (id, principal_kind, principal_id, target_kind, target, scope_kind, scope_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (principal_kind, principal_id, target_kind, target, scope_kind, scope_id) \
             DO NOTHING",
        grant.id.as_str(),
        grant.principal.kind(),
        grant.principal.id(),
        grant.target.kind(),
        grant.target.value(),
        scope_kind,
        scope_id,
    )
    .execute(executor)
    .await
    .to_domain()?;
    Ok(())
}

/// Delete every grant a principal holds at exactly `scope`, on any executor
/// (pool or transaction). Used when a member is removed from an org/project:
/// authorization is grant-driven, so the member's scoped grants must be dropped
/// atomically with their membership row, or an ex-member keeps their access.
pub async fn delete_by_principal_and_scope<'e, E>(
    executor: E,
    principal: &Principal,
    scope: &Scope,
) -> DomainResult<()>
where
    E: PgExecutor<'e>,
{
    let (scope_kind, scope_id) = match scope {
        Scope::System => (SCOPE_SYSTEM, SYSTEM_SCOPE_ID),
        Scope::Organization(id) => (SCOPE_ORGANIZATION, id.as_str()),
        Scope::Project(id) => (SCOPE_PROJECT, id.as_str()),
    };
    sqlx::query!(
        "DELETE FROM grants \
         WHERE principal_kind = $1 AND principal_id = $2 \
           AND scope_kind = $3 AND scope_id = $4",
        principal.kind(),
        principal.id(),
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
            "SELECT id, principal_kind, principal_id, target_kind, target, scope_kind, scope_id \
             FROM grants",
        )
        .fetch_all(&self.pool)
        .await
        .to_domain()?;

        rows.into_iter()
            .map(|r| {
                let principal = match r.principal_kind.as_str() {
                    PRINCIPAL_USER => Principal::User(UserId::new(r.principal_id)),
                    PRINCIPAL_APP => Principal::App(AppId::new(r.principal_id)),
                    other => {
                        return Err(DomainError::Infrastructure(format!(
                            "unknown grant principal_kind '{other}'"
                        )));
                    }
                };
                let target = match r.target_kind.as_str() {
                    TARGET_ROLE => GrantTarget::Role(RoleName::new(r.target)?),
                    TARGET_PERMISSION => GrantTarget::Permission(r.target),
                    other => {
                        return Err(DomainError::Infrastructure(format!(
                            "unknown grant target_kind '{other}'"
                        )));
                    }
                };
                let scope = match r.scope_kind.as_str() {
                    SCOPE_SYSTEM => Scope::System,
                    SCOPE_ORGANIZATION => Scope::Organization(OrganizationId::new(r.scope_id)),
                    SCOPE_PROJECT => Scope::Project(ProjectId::new(r.scope_id)),
                    other => {
                        return Err(DomainError::Infrastructure(format!(
                            "unknown grant scope_kind '{other}'"
                        )));
                    }
                };
                Ok(Grant {
                    id: r.id,
                    principal,
                    target,
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
