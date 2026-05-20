use crate::application::permission::grant::{Grant, GrantRepository, GrantScope};
use crate::domain::entities::{OrganizationId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool, Row};
use tracing::instrument;

const SCOPE_ORGANIZATION: &str = "organization";
const SCOPE_PROJECT: &str = "project";

/// Insert a grant on any executor (pool or transaction). Idempotent via the
/// `(user_id, role_name, scope_kind, scope_id)` unique constraint, so re-running
/// a signup or grant call is a no-op rather than a conflict. Shared by the
/// pool-backed repo and the atomic signup transaction.
pub async fn insert<'e, E>(executor: E, grant: &Grant) -> DomainResult<()>
where
    E: PgExecutor<'e>,
{
    let (scope_kind, scope_id) = match &grant.scope {
        GrantScope::Organization(id) => (SCOPE_ORGANIZATION, id.as_str()),
        GrantScope::Project(id) => (SCOPE_PROJECT, id.as_str()),
    };
    sqlx::query(
        "INSERT INTO permission_grants (id, user_id, role_name, scope_kind, scope_id) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (user_id, role_name, scope_kind, scope_id) DO NOTHING",
    )
    .bind(&grant.id)
    .bind(grant.user_id.as_str())
    .bind(grant.role.as_str())
    .bind(scope_kind)
    .bind(scope_id)
    .execute(executor)
    .await
    .map_err(infra)?;
    Ok(())
}

/// Persistence for explicit scoped grants (`permission_grants` table). Read once
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
        let rows = sqlx::query(
            "SELECT id, user_id, role_name, scope_kind, scope_id FROM permission_grants",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;

        rows.iter()
            .map(|r| {
                let id: String = r.try_get("id").map_err(infra)?;
                let user_id: String = r.try_get("user_id").map_err(infra)?;
                let role_name: String = r.try_get("role_name").map_err(infra)?;
                let scope_kind: String = r.try_get("scope_kind").map_err(infra)?;
                let scope_id: String = r.try_get("scope_id").map_err(infra)?;
                let scope = match scope_kind.as_str() {
                    SCOPE_ORGANIZATION => GrantScope::Organization(OrganizationId::new(scope_id)),
                    SCOPE_PROJECT => GrantScope::Project(ProjectId::new(scope_id)),
                    other => {
                        return Err(DomainError::Infrastructure(format!(
                            "unknown grant scope_kind '{other}'"
                        )));
                    }
                };
                Ok(Grant {
                    id,
                    user_id: UserId::new(user_id),
                    role: RoleName::new(role_name)?,
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
        sqlx::query("DELETE FROM permission_grants WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(infra)?;
        Ok(())
    }
}

fn infra<E: std::fmt::Display>(e: E) -> DomainError {
    DomainError::Infrastructure(e.to_string())
}
