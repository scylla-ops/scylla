use crate::application::authz::grant::ScopeKind;
use crate::application::authz::role::{Role, RoleRepository};
use crate::domain::entities::OrganizationId;
use crate::domain::errors::{DomainError, DomainResult};
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use super::error::SqlxResultExt;

const SCOPE_SYSTEM: &str = "system";
const SCOPE_ORGANIZATION: &str = "organization";
const SCOPE_PROJECT: &str = "project";

/// Persistence for role definitions (`roles` + `role_permissions`). Read at
/// `CedarPermissionService` construction and on each reload to generate the
/// per-role Cedar templates.
#[derive(Clone)]
pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for PgRoleRepository {
    #[instrument(skip(self))]
    async fn list_all(&self) -> DomainResult<Vec<Role>> {
        let rows = sqlx::query!(
            "SELECT r.id, r.key, r.name, r.description, r.scope_kind, r.owner_org_id, r.builtin, \
             COALESCE(ARRAY_AGG(rp.permission) FILTER (WHERE rp.permission IS NOT NULL), ARRAY[]::text[]) \
                 AS \"permissions!\" \
             FROM roles r \
             LEFT JOIN role_permissions rp ON rp.role_id = r.id \
             GROUP BY r.id",
        )
        .fetch_all(&self.pool)
        .await
        .to_domain()?;

        rows.into_iter()
            .map(|r| {
                let scope = match r.scope_kind.as_str() {
                    SCOPE_SYSTEM => ScopeKind::System,
                    SCOPE_ORGANIZATION => ScopeKind::Organization,
                    SCOPE_PROJECT => ScopeKind::Project,
                    other => {
                        return Err(DomainError::Infrastructure(format!(
                            "unknown role scope_kind '{other}'"
                        )));
                    }
                };
                Ok(Role {
                    id: r.id,
                    key: r.key,
                    name: r.name,
                    description: r.description,
                    scope,
                    owner_org: r.owner_org_id.map(OrganizationId::new),
                    builtin: r.builtin,
                    permissions: r.permissions,
                })
            })
            .collect()
    }
}
