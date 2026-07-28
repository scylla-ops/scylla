use crate::application::authz::grant::ScopeKind;
use crate::application::authz::role::{Role, RoleRepository};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::OrganizationId;
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
        // `role_permissions` cascades.
        sqlx::query!("DELETE FROM roles WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .to_domain()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PgRoleRepository;
    use crate::application::authz::grant::ScopeKind;
    use crate::application::authz::role::{Role, RoleRepository};
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
        use crate::domain::errors::DomainResult;
        use crate::domain::ids::UserId;
        use crate::domain::permission::Permission;
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
        // A second role, held at org scope, so the two grants group separately.
        sqlx::query!(
            "INSERT INTO roles (id, name, scope_kind, builtin) \
             VALUES ('janitor', 'Janitor', 'organization', FALSE)"
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO role_permissions (role_id, permission) VALUES ('janitor', 'deleteJob')"
        )
        .execute(&pool)
        .await
        .unwrap();
        // Alice holds `ci` at project p1 and `janitor` at org o1.
        sqlx::query!(
            "INSERT INTO grants (id, principal_kind, principal_id, role_id, scope_kind, scope_id) \
             VALUES ('g1', 'user', 'alice', 'ci', 'project', 'p1'), \
                    ('g2', 'user', 'alice', 'janitor', 'organization', 'o1')"
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

    /// Reading your own access needs no permission, while reading someone else's
    /// still requires `manageSystemGrants`. Both run against a permission service
    /// that denies everything, so the split is what is under test, not the stub.
    #[sqlx::test(migrations = "../../migrations")]
    async fn my_permissions_needs_no_permission_unlike_the_admin_view(pool: PgPool) {
        use crate::application::PermissionService;
        use crate::application::authz::policy::PolicyControl;
        use crate::application::authz::{Principal, RoleUseCases, Scope};
        use crate::application::caller::{CallerContext, ServiceIdentity};
        use crate::domain::errors::{DomainError, DomainResult};
        use crate::domain::ids::UserId;
        use crate::domain::permission::Permission;
        use crate::infrastructure::persistence::postgres::PgGrantRepository;
        use std::sync::Arc;

        struct DenyAll;
        #[async_trait::async_trait]
        impl PermissionService for DenyAll {
            async fn check(&self, _c: &CallerContext, _p: Permission) -> DomainResult<()> {
                Err(DomainError::Forbidden("denied".to_string()))
            }
        }
        struct NoopPolicy;
        #[async_trait::async_trait]
        impl PolicyControl for NoopPolicy {
            async fn reload(&self) -> DomainResult<()> {
                Ok(())
            }
        }

        sqlx::query!(
            "INSERT INTO grants (id, principal_kind, principal_id, role_id, scope_kind, scope_id) \
             VALUES ('g1', 'user', 'alice', 'organization-admin', 'organization', 'o1')"
        )
        .execute(&pool)
        .await
        .unwrap();

        let uc = RoleUseCases::new(
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool)),
            Arc::new(DenyAll),
            Arc::new(NoopPolicy),
        );

        // Alice reads her own access even though every permission check fails.
        let alice = CallerContext::User(UserId::new("alice"));
        let scopes = uc.my_permissions(&alice).await.expect("own permissions");
        assert_eq!(scopes.len(), 1);
        assert!(matches!(&scopes[0].scope, Scope::Organization(o) if o.as_str() == "o1"));
        assert!(scopes[0].full_control, "organization-admin confers '*'");

        // A user with no grants gets an empty list, not an error.
        let bob = CallerContext::User(UserId::new("bob"));
        assert!(uc.my_permissions(&bob).await.unwrap().is_empty());

        // The admin view over another principal is still gated.
        assert!(
            uc.effective_permissions(&alice, &Principal::User(UserId::new("bob")))
                .await
                .is_err(),
            "reading another principal must still require manageSystemGrants",
        );

        // A service acts as the system and holds no grants: refused, not empty.
        let service = CallerContext::Service(ServiceIdentity::recorder());
        assert!(uc.my_permissions(&service).await.is_err());
    }
}
