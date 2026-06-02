use crate::application::authz::grant::ScopeKind;
use crate::application::authz::role::{
    DefaultRoleBindingRepository, DefaultRoleSlot, Role, RoleRepository,
};
use crate::domain::entities::OrganizationId;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::role::name::RoleName;
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

/// Reads the configurable default-role pointers (`default_role_bindings`).
#[derive(Clone)]
pub struct PgDefaultRoleBindingRepository {
    pool: PgPool,
}

impl PgDefaultRoleBindingRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DefaultRoleBindingRepository for PgDefaultRoleBindingRepository {
    #[instrument(skip(self))]
    async fn role_for_slot(&self, slot: DefaultRoleSlot) -> DomainResult<Option<RoleName>> {
        // Join through `roles` so a binding whose role no longer exists reads as
        // `None` (the caller then falls back to the builtin).
        let row = sqlx::query!(
            "SELECT b.role_id FROM default_role_bindings b \
             JOIN roles r ON r.id = b.role_id \
             WHERE b.slot = $1",
            slot.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .to_domain()?;

        row.map(|r| RoleName::new(r.role_id)).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::PgDefaultRoleBindingRepository;
    use crate::application::authz::role::{
        DefaultRoleBindingRepository, DefaultRoleSlot, resolve_default_role,
    };
    use sqlx::PgPool;

    #[sqlx::test(migrations = "../../migrations")]
    async fn default_binding_resolves_to_seeded_builtin(pool: PgPool) {
        let repo = PgDefaultRoleBindingRepository::new(pool);
        let role = repo
            .role_for_slot(DefaultRoleSlot::OrgCreation)
            .await
            .unwrap();
        assert_eq!(role.unwrap().as_str(), "organization-admin");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn rebound_slot_resolves_to_the_custom_role(pool: PgPool) {
        // Seed a custom org-scoped role and rebind org_creation to it.
        sqlx::query!(
            "INSERT INTO roles (id, name, scope_kind, builtin) \
             VALUES ('custom-1', 'Custom', 'organization', FALSE)"
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query!(
            "UPDATE default_role_bindings SET role_id = 'custom-1' WHERE slot = 'org_creation'"
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = PgDefaultRoleBindingRepository::new(pool);
        let role = resolve_default_role(&repo, DefaultRoleSlot::OrgCreation)
            .await
            .unwrap();
        assert_eq!(role.as_str(), "custom-1");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn missing_binding_falls_back_to_builtin(pool: PgPool) {
        sqlx::query!("DELETE FROM default_role_bindings WHERE slot = 'org_creation'")
            .execute(&pool)
            .await
            .unwrap();

        let repo = PgDefaultRoleBindingRepository::new(pool);
        // The binding is gone, so the repo reads `None`...
        assert!(
            repo.role_for_slot(DefaultRoleSlot::OrgCreation)
                .await
                .unwrap()
                .is_none()
        );
        // ...and resolution falls back to the slot's builtin role.
        let role = resolve_default_role(&repo, DefaultRoleSlot::OrgCreation)
            .await
            .unwrap();
        assert_eq!(role.as_str(), "organization-admin");
    }
}
