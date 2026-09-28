use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::OrganizationId;
use crate::domain::role::{RoleDescription, RoleDisplayName};
use async_trait::async_trait;
use scylla_auth::authz::{Role, RoleRepository, ScopeKind};
use sqlx::{PgConnection, PgPool};
use tracing::instrument;

use super::error::{DbFieldExt, SqlxResultExt};

const SCOPE_SYSTEM: &str = "system";
const SCOPE_ORGANIZATION: &str = "organization";
const SCOPE_PROJECT: &str = "project";

#[derive(Clone)]
pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn select(&self, id: Option<&str>) -> DomainResult<Vec<Role>> {
        let rows = sqlx::query!(
            "SELECT r.id, r.key, r.name, r.description, r.scope_kind, r.owner_org_id, r.builtin, \
             COALESCE(ARRAY_AGG(rp.permission) FILTER (WHERE rp.permission IS NOT NULL), ARRAY[]::text[]) \
                 AS \"permissions!\" \
             FROM roles r \
             LEFT JOIN role_permissions rp ON rp.role_id = r.id \
             WHERE $1::text IS NULL OR r.id = $1 \
             GROUP BY r.id",
            id,
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
                    name: RoleDisplayName::new(r.name).db_field("role name")?,
                    description: RoleDescription::new(r.description)
                        .db_field("role description")?,
                    scope,
                    owner_org: r.owner_org_id.map(OrganizationId::new),
                    builtin: r.builtin,
                    permissions: r.permissions,
                })
            })
            .collect()
    }
}

async fn insert_permissions(tx: &mut PgConnection, role: &Role) -> DomainResult<()> {
    sqlx::query!(
        "INSERT INTO role_permissions (role_id, permission) SELECT $1, unnest($2::text[])",
        role.id,
        &role.permissions,
    )
    .execute(tx)
    .await
    .to_domain()?;
    Ok(())
}

#[async_trait]
impl RoleRepository for PgRoleRepository {
    #[instrument(skip(self))]
    async fn list_all(&self) -> DomainResult<Vec<Role>> {
        self.select(None).await
    }

    #[instrument(skip(self))]
    async fn get(&self, id: &str) -> DomainResult<Option<Role>> {
        Ok(self.select(Some(id)).await?.pop())
    }

    #[instrument(skip_all, fields(role_id = %role.id))]
    async fn create(&self, role: &Role) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;
        sqlx::query!(
            "INSERT INTO roles (id, key, name, description, scope_kind, owner_org_id, builtin) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            role.id,
            role.key.as_deref(),
            role.name.as_str(),
            role.description.as_str(),
            role.scope.as_str(),
            role.owner_org.as_ref().map(OrganizationId::as_str),
            role.builtin,
        )
        .execute(&mut *tx)
        .await
        .to_domain()?;
        insert_permissions(&mut tx, role).await?;
        tx.commit().await.to_domain()
    }

    #[instrument(skip_all, fields(role_id = %role.id))]
    async fn update(&self, role: &Role) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.to_domain()?;
        sqlx::query!(
            "UPDATE roles SET name = $2, description = $3, updated_at = NOW() WHERE id = $1",
            role.id,
            role.name.as_str(),
            role.description.as_str(),
        )
        .execute(&mut *tx)
        .await
        .to_domain()?;
        sqlx::query!("DELETE FROM role_permissions WHERE role_id = $1", role.id)
            .execute(&mut *tx)
            .await
            .to_domain()?;
        insert_permissions(&mut tx, role).await?;
        tx.commit().await.to_domain()
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &str) -> DomainResult<()> {
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
    use crate::domain::errors::DomainError;
    use crate::domain::role::{RoleDescription, RoleDisplayName, RoleName};
    use scylla_auth::authz::{Role, RoleRepository, ScopeKind};
    use sqlx::PgPool;

    #[sqlx::test(migrations = "../../migrations")]
    async fn an_organization_viewer_lists_the_members_of_its_projects(pool: PgPool) {
        let viewer = PgRoleRepository::new(pool)
            .get(scylla_auth::authz::ORGANIZATION_VIEWER_ROLE)
            .await
            .unwrap()
            .unwrap();
        assert!(
            viewer
                .permissions
                .contains(&"listProjectMembers".to_string())
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn role_crud_round_trip(pool: PgPool) {
        let repo = PgRoleRepository::new(pool);
        let role = Role::new_custom(
            RoleDisplayName::new("CI Runner").unwrap(),
            RoleDescription::new("reads and runs pipelines").unwrap(),
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
        assert_eq!(got.name.as_str(), "CI Runner");
        assert_eq!(got.permissions.len(), 2);
        assert!(!got.builtin);
        assert!(got.key.is_none());

        let mut updated = got.clone();
        updated.name = RoleDisplayName::new("CI").unwrap();
        updated.permissions = vec!["readPipeline".into()];
        repo.update(&updated).await.unwrap();
        let got = repo.get(&id).await.unwrap().unwrap();
        assert_eq!(got.name.as_str(), "CI");
        assert_eq!(got.permissions, vec!["readPipeline".to_string()]);

        repo.delete(&id).await.unwrap();
        assert!(repo.get(&id).await.unwrap().is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn a_display_name_is_unique_across_roles(pool: PgPool) {
        let repo = PgRoleRepository::new(pool);
        let shadow = Role::new_custom(
            RoleDisplayName::new("System Admin").unwrap(),
            RoleDescription::new("").unwrap(),
            ScopeKind::Project,
            vec!["readPipeline".into()],
        );
        let err = repo.create(&shadow).await.unwrap_err();
        assert!(
            matches!(&err, DomainError::Conflict(m) if m == "Role name already exists"),
            "{err}"
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn effective_permissions_resolves_roles_and_direct_grants(pool: PgPool) {
        use crate::domain::caller::{CallerContext, ServiceIdentity};
        use crate::domain::errors::DomainResult;
        use crate::postgres::PgGrantRepository;
        use crate::test_support::prelude::*;
        use scylla_auth::authz::{Grant, GrantRepository, PolicyControl, Principal, Scope};
        use scylla_core::application::role::{GetEffectivePermissions, RoleUseCases};
        use scylla_core::test_support::authz::{RecordingPermissionService, actions};
        use std::sync::Arc;

        struct NoopPolicy;
        #[async_trait::async_trait]
        impl PolicyControl for NoopPolicy {
            async fn reload(&self) -> DomainResult<()> {
                Ok(())
            }
        }

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
        let org = seed_org(&pool, "o1").await;
        let project = seed_project(&pool, &org, "p1").await;
        let alice = seed_user(&pool, "alice").await;
        let grants = PgGrantRepository::new(pool.clone());
        for (role, scope) in [
            ("ci", Scope::Project(project.id().clone())),
            ("janitor", Scope::Organization(org.id().clone())),
        ] {
            grants
                .create(&Grant::new(
                    Principal::User(alice.id().clone()),
                    RoleName::new(role).unwrap(),
                    scope,
                ))
                .await
                .unwrap();
        }

        let uc = RoleUseCases::new(
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool)),
            Arc::new(NoopPolicy),
        );
        let actions = actions(Arc::new(RecordingPermissionService::new()));
        let caller = CallerContext::Service(ServiceIdentity::recorder());
        let scopes = actions
            .run(
                &uc,
                &caller,
                GetEffectivePermissions {
                    principal: Principal::User(alice.id().clone()),
                },
            )
            .await
            .unwrap();

        assert_eq!(scopes.len(), 2);
        let on_project = scopes
            .iter()
            .find(|s| matches!(&s.scope, Scope::Project(p) if p == project.id()))
            .expect("project p1 scope");
        assert!(!on_project.full_control);
        assert!(on_project.permissions.contains(&"readPipeline".to_string()));
        assert!(on_project.permissions.contains(&"runPipeline".to_string()));
        let on_org = scopes
            .iter()
            .find(|s| matches!(&s.scope, Scope::Organization(o) if o == org.id()))
            .expect("org o1 scope");
        assert_eq!(on_org.permissions, vec!["deleteJob".to_string()]);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn my_permissions_needs_no_permission_unlike_the_admin_view(pool: PgPool) {
        use crate::domain::caller::{CallerContext, ServiceIdentity};
        use crate::domain::errors::DomainResult;
        use crate::domain::ids::UserId;
        use crate::postgres::PgGrantRepository;
        use crate::test_support::prelude::*;
        use scylla_auth::authz::{
            Grant, GrantRepository, ORGANIZATION_ADMIN_ROLE, PolicyControl, Principal, Scope,
        };
        use scylla_core::application::role::{
            GetEffectivePermissions, GetMyPermissions, RoleUseCases,
        };
        use scylla_core::test_support::authz::{DenyingPermissionService, actions};
        use std::sync::Arc;

        struct NoopPolicy;
        #[async_trait::async_trait]
        impl PolicyControl for NoopPolicy {
            async fn reload(&self) -> DomainResult<()> {
                Ok(())
            }
        }

        let org = seed_org(&pool, "o1").await;
        let alice = seed_user(&pool, "alice").await;
        PgGrantRepository::new(pool.clone())
            .create(&Grant::new(
                Principal::User(alice.id().clone()),
                RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap(),
                Scope::Organization(org.id().clone()),
            ))
            .await
            .unwrap();

        let uc = RoleUseCases::new(
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool)),
            Arc::new(NoopPolicy),
        );
        let actions = actions(Arc::new(DenyingPermissionService::new()));

        let alice = CallerContext::User(alice.id().clone());
        let scopes = actions
            .run(&uc, &alice, GetMyPermissions)
            .await
            .expect("own permissions");
        assert_eq!(scopes.len(), 1);
        assert!(matches!(&scopes[0].scope, Scope::Organization(o) if o == org.id()));
        assert!(scopes[0].full_control, "organization-admin confers '*'");

        let bob = CallerContext::User(UserId::new("bob"));
        let scopes = actions.run(&uc, &bob, GetMyPermissions).await.unwrap();
        assert!(scopes.is_empty());

        assert!(
            actions
                .run(
                    &uc,
                    &alice,
                    GetEffectivePermissions {
                        principal: Principal::User(UserId::new("bob")),
                    },
                )
                .await
                .is_err(),
            "reading another principal must still require manageSystemGrants",
        );

        let service = CallerContext::Service(ServiceIdentity::recorder());
        let refused = actions.run(&uc, &service, GetMyPermissions).await;
        assert!(refused.is_err());
    }
}
