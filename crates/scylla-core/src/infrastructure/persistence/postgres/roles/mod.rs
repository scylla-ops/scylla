use crate::application::authz::grant::ScopeKind;
use crate::application::authz::role::{
    DefaultRoleBindingRepository, DefaultRoleSlot, Role, RoleRepository,
};
use crate::domain::entities::OrganizationId;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::role::RoleName;
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

    #[instrument(skip(self))]
    async fn get(&self, id: &str) -> DomainResult<Option<Role>> {
        let row = sqlx::query!(
            "SELECT r.id, r.key, r.name, r.description, r.scope_kind, r.owner_org_id, r.builtin, \
             COALESCE(ARRAY_AGG(rp.permission) FILTER (WHERE rp.permission IS NOT NULL), ARRAY[]::text[]) \
                 AS \"permissions!\" \
             FROM roles r \
             LEFT JOIN role_permissions rp ON rp.role_id = r.id \
             WHERE r.id = $1 \
             GROUP BY r.id",
            id,
        )
        .fetch_optional(&self.pool)
        .await
        .to_domain()?;

        row.map(|r| {
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
        .transpose()
    }

    #[instrument(skip(self, role), fields(role_id = %role.id))]
    async fn create(&self, role: &Role) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;
        sqlx::query!(
            "INSERT INTO roles (id, key, name, description, scope_kind, owner_org_id, builtin) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            role.id,
            role.key.as_deref(),
            role.name,
            role.description,
            role.scope.as_str(),
            role.owner_org.as_ref().map(OrganizationId::as_str),
            role.builtin,
        )
        .execute(&mut *tx)
        .await
        .to_domain()?;
        for permission in &role.permissions {
            sqlx::query!(
                "INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2)",
                role.id,
                permission,
            )
            .execute(&mut *tx)
            .await
            .to_domain()?;
        }
        tx.commit().await.to_domain()
    }

    #[instrument(skip(self, role), fields(role_id = %role.id))]
    async fn update(&self, role: &Role) -> DomainResult<()> {
        // The role's scope kind / builtin / owner are immutable; only name,
        // description and the permission set change. Replace the permission set
        // wholesale inside the transaction.
        let mut tx = self.pool.begin().await.to_domain()?;
        sqlx::query!(
            "UPDATE roles SET name = $2, description = $3, updated_at = NOW() WHERE id = $1",
            role.id,
            role.name,
            role.description,
        )
        .execute(&mut *tx)
        .await
        .to_domain()?;
        sqlx::query!("DELETE FROM role_permissions WHERE role_id = $1", role.id)
            .execute(&mut *tx)
            .await
            .to_domain()?;
        for permission in &role.permissions {
            sqlx::query!(
                "INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2)",
                role.id,
                permission,
            )
            .execute(&mut *tx)
            .await
            .to_domain()?;
        }
        tx.commit().await.to_domain()
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &str) -> DomainResult<()> {
        // `role_permissions` cascades; a role still pointed at by a
        // default-role binding is blocked by that FK's ON DELETE RESTRICT.
        sqlx::query!("DELETE FROM roles WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .to_domain()?;
        Ok(())
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
    use super::{PgDefaultRoleBindingRepository, PgRoleRepository};
    use crate::application::authz::grant::ScopeKind;
    use crate::application::authz::role::{
        DefaultRoleBindingRepository, DefaultRoleSlot, Role, RoleRepository, resolve_default_role,
    };
    use sqlx::PgPool;

    #[sqlx::test(migrations = "../../migrations")]
    async fn role_crud_round_trip(pool: PgPool) {
        let repo = PgRoleRepository::new(pool);
        let role = Role::new_custom(
            "CI Runner".into(),
            "reads and runs pipelines".into(),
            ScopeKind::Project,
            vec!["readPipeline".into(), "runPipeline".into()],
        );
        let id = role.id.clone();
        repo.create(&role).await.unwrap();

        let got = repo
            .get(&id)
            .await
            .unwrap()
            .expect("role exists after create");
        assert_eq!(got.name, "CI Runner");
        assert_eq!(got.permissions.len(), 2);
        assert!(!got.builtin);
        assert!(got.key.is_none());

        let mut updated = got.clone();
        updated.name = "CI".into();
        updated.permissions = vec!["readPipeline".into()];
        repo.update(&updated).await.unwrap();
        let got = repo.get(&id).await.unwrap().unwrap();
        assert_eq!(got.name, "CI");
        assert_eq!(got.permissions, vec!["readPipeline".to_string()]);

        repo.delete(&id).await.unwrap();
        assert!(repo.get(&id).await.unwrap().is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn effective_permissions_resolves_roles_and_direct_grants(pool: PgPool) {
        use crate::application::PermissionService;
        use crate::application::authz::policy::PolicyControl;
        use crate::application::authz::{Principal, RoleUseCases, Scope};
        use crate::application::caller::{CallerContext, ServiceIdentity};
        use crate::domain::entities::UserId;
        use crate::domain::errors::DomainResult;
        use crate::domain::value_objects::permission::Permission;
        use crate::infrastructure::persistence::postgres::PgGrantRepository;
        use std::sync::Arc;

        struct AllowAll;
        #[async_trait::async_trait]
        impl PermissionService for AllowAll {
            async fn check(&self, _c: &CallerContext, _p: Permission) -> DomainResult<()> {
                Ok(())
            }
        }
        struct NoopPolicy;
        #[async_trait::async_trait]
        impl PolicyControl for NoopPolicy {
            async fn validate_policy(&self, _t: &str) -> DomainResult<()> {
                Ok(())
            }
            async fn reload(&self) -> DomainResult<()> {
                Ok(())
            }
        }

        // A custom project-scoped role with two permissions.
        sqlx::query!(
            "INSERT INTO roles (id, name, scope_kind, builtin) VALUES ('ci', 'CI', 'project', FALSE)"
        )
        .execute(&pool)
        .await
        .unwrap();
        for p in ["readPipeline", "runPipeline"] {
            sqlx::query!(
                "INSERT INTO role_permissions (role_id, permission) VALUES ('ci', $1)",
                p
            )
            .execute(&pool)
            .await
            .unwrap();
        }
        // Alice holds the role at project p1, and a direct deleteJob at org o1.
        sqlx::query!(
            "INSERT INTO grants (id, principal_kind, principal_id, target_kind, target, scope_kind, scope_id) \
             VALUES ('g1', 'user', 'alice', 'role', 'ci', 'project', 'p1'), \
                    ('g2', 'user', 'alice', 'permission', 'deleteJob', 'organization', 'o1')"
        )
        .execute(&pool)
        .await
        .unwrap();

        let uc = RoleUseCases::new(
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool)),
            Arc::new(AllowAll),
            Arc::new(NoopPolicy),
        );
        let caller = CallerContext::Service(ServiceIdentity::recorder());
        let scopes = uc
            .effective_permissions(&caller, &Principal::User(UserId::new("alice")))
            .await
            .unwrap();

        assert_eq!(scopes.len(), 2);
        let project = scopes
            .iter()
            .find(|s| matches!(&s.scope, Scope::Project(p) if p.as_str() == "p1"))
            .expect("project p1 scope");
        assert!(!project.full_control);
        assert!(project.permissions.contains(&"readPipeline".to_string()));
        assert!(project.permissions.contains(&"runPipeline".to_string()));
        let org = scopes
            .iter()
            .find(|s| matches!(&s.scope, Scope::Organization(o) if o.as_str() == "o1"))
            .expect("org o1 scope");
        assert_eq!(org.permissions, vec!["deleteJob".to_string()]);
    }

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
