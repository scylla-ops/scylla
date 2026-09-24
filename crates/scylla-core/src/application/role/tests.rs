//! The role's actions through the engine, on stub ports.

use super::*;
use crate::application::PermissionAuthorizer;
use crate::domain::caller::ServiceIdentity;
use crate::domain::ids::{OrganizationId, ProjectId, UserId};
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService};
use async_trait::async_trait;
use scylla_auth::authz::{Grant, PermissionService, Role, ScopeKind};
use scylla_extension::{Actions, Hooks};
use std::sync::Mutex;

#[derive(Default)]
struct StubRoles {
    rows: Mutex<Vec<Role>>,
}

#[async_trait]
impl RoleRepository for StubRoles {
    async fn list_all(&self) -> DomainResult<Vec<Role>> {
        Ok(self.rows.lock().unwrap().clone())
    }
    async fn get(&self, id: &str) -> DomainResult<Option<Role>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.id == id)
            .cloned())
    }
    async fn create(&self, role: &Role) -> DomainResult<()> {
        self.rows.lock().unwrap().push(role.clone());
        Ok(())
    }
    async fn update(&self, role: &Role) -> DomainResult<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(row) = rows.iter_mut().find(|r| r.id == role.id) {
            *row = role.clone();
        }
        Ok(())
    }
    async fn delete(&self, id: &str) -> DomainResult<()> {
        self.rows.lock().unwrap().retain(|r| r.id != id);
        Ok(())
    }
}

struct StubGrants(Vec<Grant>);

#[async_trait]
impl GrantRepository for StubGrants {
    async fn list_all(&self) -> DomainResult<Vec<Grant>> {
        Ok(self.0.clone())
    }
    async fn create(&self, _: &Grant) -> DomainResult<()> {
        Ok(())
    }
    async fn delete(&self, _: &str) -> DomainResult<()> {
        Ok(())
    }
    async fn revoke_all(&self, _: &Principal, _: &Scope) -> DomainResult<u64> {
        Ok(0)
    }
}

#[derive(Default)]
struct CountingPolicy {
    reloads: Mutex<usize>,
}

#[async_trait]
impl PolicyControl for CountingPolicy {
    async fn reload(&self) -> DomainResult<()> {
        *self.reloads.lock().unwrap() += 1;
        Ok(())
    }
}

struct Lab {
    actions: Actions,
    uc: RoleUseCases,
    roles: Arc<StubRoles>,
    policy: Arc<CountingPolicy>,
}

fn lab(permissions: Arc<dyn PermissionService>, roles: Vec<Role>, grants: Vec<Grant>) -> Lab {
    let roles = Arc::new(StubRoles {
        rows: Mutex::new(roles),
    });
    let policy = Arc::new(CountingPolicy::default());
    Lab {
        actions: Actions::new(
            Arc::new(PermissionAuthorizer::new(permissions)),
            Arc::new(Hooks::new()),
        ),
        uc: RoleUseCases::new(roles.clone(), Arc::new(StubGrants(grants)), policy.clone()),
        roles,
        policy,
    }
}

fn alice() -> CallerContext {
    CallerContext::User(UserId::new("alice"))
}

fn role(id: &str, scope: ScopeKind, builtin: bool, permissions: &[&str]) -> Role {
    Role {
        id: id.to_string(),
        key: builtin.then(|| id.to_string()),
        name: id.to_string(),
        description: String::new(),
        scope,
        owner_org: None,
        builtin,
        permissions: permissions.iter().map(ToString::to_string).collect(),
    }
}

fn create(permissions: &[&str]) -> CreateRole {
    CreateRole {
        name: "CI Runner".to_string(),
        description: String::new(),
        scope: ScopeKind::Project,
        permissions: permissions.iter().map(ToString::to_string).collect(),
    }
}

#[tokio::test]
async fn a_create_checks_manage_roles_then_stores_and_reloads() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone(), vec![], vec![]);

    let created = lab
        .actions
        .run(&lab.uc, &alice(), create(&["readPipeline"]))
        .await
        .unwrap();

    assert_eq!(permissions.permissions(), vec![Permission::ManageRoles]);
    assert!(!created.builtin);
    assert_eq!(lab.roles.rows.lock().unwrap().as_slice(), [created]);
    assert_eq!(*lab.policy.reloads.lock().unwrap(), 1);
}

#[tokio::test]
async fn a_denied_create_never_stores() {
    let lab = lab(Arc::new(DenyingPermissionService::new()), vec![], vec![]);

    let err = lab
        .actions
        .run(&lab.uc, &alice(), create(&["readPipeline"]))
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.roles.rows.lock().unwrap().is_empty());
    assert_eq!(*lab.policy.reloads.lock().unwrap(), 0);
}

#[tokio::test]
async fn a_permission_out_of_the_role_scope_is_refused_before_anything_is_stored() {
    let lab = lab(Arc::new(RecordingPermissionService::new()), vec![], vec![]);

    let err = lab
        .actions
        .run(&lab.uc, &alice(), create(&["createOrganization"]))
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::Validation(_)));
    assert!(lab.roles.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn an_update_validates_against_the_stored_scope_and_rewrites_the_role() {
    let lab = lab(
        Arc::new(RecordingPermissionService::new()),
        vec![role("ci", ScopeKind::Project, false, &["readPipeline"])],
        vec![],
    );
    let update = |permissions: &[&str]| UpdateRole {
        id: "ci".to_string(),
        name: "CI".to_string(),
        description: "runs the builds".to_string(),
        permissions: permissions.iter().map(ToString::to_string).collect(),
    };

    let err = lab
        .actions
        .run(&lab.uc, &alice(), update(&["createProject"]))
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::Validation(_)));

    let updated = lab
        .actions
        .run(&lab.uc, &alice(), update(&["runPipeline"]))
        .await
        .unwrap();
    assert_eq!(updated.name, "CI");
    assert_eq!(updated.permissions, vec!["runPipeline".to_string()]);
    assert_eq!(lab.roles.rows.lock().unwrap().as_slice(), [updated]);
}

#[tokio::test]
async fn a_missing_role_is_not_found() {
    let lab = lab(Arc::new(RecordingPermissionService::new()), vec![], vec![]);

    let err = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            GetRole {
                id: "ghost".to_string(),
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::NotFound { .. }));
}

#[tokio::test]
async fn a_builtin_or_granted_role_cannot_be_deleted() {
    let granted = Grant::new(
        Principal::User(UserId::new("bob")),
        RoleName::new("ci").unwrap(),
        Scope::Project(ProjectId::new("p1")),
    );
    let lab = lab(
        Arc::new(RecordingPermissionService::new()),
        vec![
            role("organization-admin", ScopeKind::Organization, true, &["*"]),
            role("ci", ScopeKind::Project, false, &["readPipeline"]),
            role("unused", ScopeKind::Project, false, &["readPipeline"]),
        ],
        vec![granted],
    );
    let delete = |id: &str| DeleteRole { id: id.to_string() };

    for id in ["organization-admin", "ci"] {
        let err = lab
            .actions
            .run(&lab.uc, &alice(), delete(id))
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::BusinessRule(_)), "{id}");
    }

    let deleted = lab
        .actions
        .run(&lab.uc, &alice(), delete("unused"))
        .await
        .unwrap();
    assert_eq!(deleted.last_state().id, "unused");
    assert_eq!(lab.roles.rows.lock().unwrap().len(), 2);
    assert_eq!(*lab.policy.reloads.lock().unwrap(), 1);
}

#[tokio::test]
async fn effective_permissions_group_the_grants_by_scope_behind_manage_system_grants() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let alice_principal = Principal::User(UserId::new("alice"));
    let lab = lab(
        permissions.clone(),
        vec![
            role(
                "ci",
                ScopeKind::Project,
                false,
                &["readPipeline", "runPipeline"],
            ),
            role("janitor", ScopeKind::Organization, false, &["deleteJob"]),
            role("organization-admin", ScopeKind::Organization, true, &["*"]),
        ],
        vec![
            Grant::new(
                alice_principal.clone(),
                RoleName::new("ci").unwrap(),
                Scope::Project(ProjectId::new("p1")),
            ),
            Grant::new(
                alice_principal.clone(),
                RoleName::new("janitor").unwrap(),
                Scope::Organization(OrganizationId::new("o1")),
            ),
            Grant::new(
                alice_principal.clone(),
                RoleName::new("organization-admin").unwrap(),
                Scope::Organization(OrganizationId::new("o2")),
            ),
        ],
    );

    let scopes = lab
        .actions
        .run(
            &lab.uc,
            &CallerContext::Service(ServiceIdentity::recorder()),
            GetEffectivePermissions {
                principal: alice_principal,
            },
        )
        .await
        .unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::ManageSystemGrants]
    );
    assert_eq!(scopes.len(), 3);
    assert_eq!(
        scopes[0].permissions,
        vec!["readPipeline".to_string(), "runPipeline".to_string()]
    );
    assert_eq!(scopes[1].permissions, vec!["deleteJob".to_string()]);
    assert!(scopes[2].full_control);
    assert!(scopes[2].permissions.is_empty());
}

#[tokio::test]
async fn my_permissions_asks_for_no_permission_and_refuses_a_non_principal() {
    let lab = lab(
        Arc::new(DenyingPermissionService::new()),
        vec![role(
            "organization-admin",
            ScopeKind::Organization,
            true,
            &["*"],
        )],
        vec![Grant::new(
            Principal::User(UserId::new("alice")),
            RoleName::new("organization-admin").unwrap(),
            Scope::Organization(OrganizationId::new("o1")),
        )],
    );

    let scopes = lab.uc.my_permissions(&alice()).await.unwrap();
    assert_eq!(scopes.len(), 1);
    assert!(scopes[0].full_control);

    let err = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            GetEffectivePermissions {
                principal: Principal::User(UserId::new("bob")),
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::Forbidden(_)));

    assert!(
        lab.uc
            .my_permissions(&CallerContext::Service(ServiceIdentity::recorder()))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn the_vocabulary_is_the_permission_catalog_behind_manage_roles() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone(), vec![], vec![]);

    let vocabulary = lab
        .actions
        .run(&lab.uc, &alice(), ListAuthzVocabulary)
        .await
        .unwrap();

    assert_eq!(
        vocabulary.len(),
        crate::domain::permission::PERMISSION_CATALOG.len()
    );
    assert_eq!(permissions.permissions(), vec![Permission::ManageRoles]);
}
