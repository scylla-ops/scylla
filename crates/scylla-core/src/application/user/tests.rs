//! The user's actions through the engine, on stub ports.

use super::*;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::UserId;
use crate::domain::ids::{OrganizationId, ProjectId};
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use crate::domain::user::{Email, Password, User, Username};
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService, actions};
use crate::test_support::stubs::{CountingPolicy, StubGrants, StubHash, alice};
use crate::test_support::users::user;
use async_trait::async_trait;
use scylla_auth::authz::{
    Grant, ORGANIZATION_ADMIN_ROLE, PROJECT_ADMIN_ROLE, PermissionService, Principal,
    SYSTEM_ADMIN_ROLE, Scope,
};
use scylla_extension::Actions;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubUsers {
    rows: Mutex<HashMap<UserId, User>>,
}

#[async_trait]
impl UserRepository for StubUsers {
    async fn create(&self, user: &User) -> DomainResult<User> {
        let mut rows = self.rows.lock().unwrap();
        if rows
            .values()
            .any(|u| u.username() == user.username() && u.id() != user.id())
        {
            return Err(DomainError::conflict("Username already exists"));
        }
        rows.insert(user.id().clone(), user.clone());
        Ok(user.clone())
    }
    async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("User", id))
    }
    async fn find_by_ids(&self, ids: &[UserId]) -> DomainResult<Vec<User>> {
        let rows = self.rows.lock().unwrap();
        Ok(ids
            .iter()
            .rev()
            .filter_map(|id| rows.get(id).cloned())
            .collect())
    }
    async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
        self.rows
            .lock()
            .unwrap()
            .values()
            .find(|u| u.username() == username)
            .cloned()
            .ok_or_else(|| DomainError::not_found("User", username))
    }
    async fn find_by_email(&self, email: &Email) -> DomainResult<User> {
        Err(DomainError::not_found("User", email))
    }
    async fn update(&self, user: &User) -> DomainResult<User> {
        self.create(user).await
    }
    async fn delete(&self, user: &User) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(user.id());
        Ok(())
    }
    async fn list_all(&self, _: Option<&PaginationParams>) -> DomainResult<PaginatedResult<User>> {
        Ok(PaginatedResult::new(
            Vec::new(),
            &PaginationParams::default(),
            0,
        ))
    }
}

struct Lab {
    actions: Actions,
    uc: UserUseCases,
    users: Arc<StubUsers>,
    policy: Arc<CountingPolicy>,
}

impl Lab {
    async fn create(&self, username: &str) -> DomainResult<User> {
        self.actions.run(&self.uc, &alice(), create(username)).await
    }
}

fn lab(permissions: Arc<dyn PermissionService>) -> Lab {
    lab_with(permissions, Vec::new())
}

fn lab_with(permissions: Arc<dyn PermissionService>, grants: Vec<Grant>) -> Lab {
    let users = Arc::new(StubUsers::default());
    let policy = Arc::new(CountingPolicy::default());
    Lab {
        actions: actions(permissions),
        uc: UserUseCases::new(
            users.clone(),
            Arc::new(StubGrants::new(grants)),
            Arc::new(StubHash::passwords()),
            policy.clone(),
        ),
        users,
        policy,
    }
}

fn create(username: &str) -> CreateUser {
    CreateUser {
        username: Username::new(username).unwrap(),
        email: None,
        password: Password::new("SecurePass123!").unwrap(),
    }
}

#[tokio::test]
async fn a_create_checks_the_permission_then_stores_the_hashed_user() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());

    let user = lab.create("bob").await.unwrap();

    assert_eq!(permissions.permissions(), vec![Permission::CreateUser]);
    assert!(lab.users.rows.lock().unwrap().contains_key(user.id()));
    assert_eq!(lab.policy.reloads(), 0);
}

#[tokio::test]
async fn a_denied_caller_writes_nothing() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));

    let err = lab.create("bob").await.unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.users.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_taken_username_is_a_conflict_and_writes_nothing() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    lab.create("bob").await.unwrap();

    let err = lab.create("bob").await.unwrap_err();

    assert!(matches!(err, DomainError::Conflict(_)));
    assert_eq!(lab.users.rows.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn an_update_stages_the_change_and_persists_it() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create("old").await.unwrap();

    let updated = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            UpdateUser {
                id: created.id().clone(),
                username: Some(Username::new("new").unwrap()),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.username().as_str(), "new");
    assert_eq!(
        lab.users.rows.lock().unwrap()[created.id()]
            .username()
            .as_str(),
        "new"
    );
    assert_eq!(
        permissions.permissions()[1],
        Permission::UpdateUser(created.id().clone())
    );
}

#[tokio::test]
async fn an_update_to_its_own_username_is_not_a_conflict() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    let created = lab.create("bob").await.unwrap();

    lab.actions
        .run(
            &lab.uc,
            &alice(),
            UpdateUser {
                id: created.id().clone(),
                username: Some(Username::new("bob").unwrap()),
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn an_update_to_a_taken_username_is_a_conflict() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    lab.create("taken").await.unwrap();
    let created = lab.create("bob").await.unwrap();

    let err = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            UpdateUser {
                id: created.id().clone(),
                username: Some(Username::new("taken").unwrap()),
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::Conflict(_)));
}

#[tokio::test]
async fn a_delete_returns_the_tombstone_and_reloads_the_policies() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create("gone").await.unwrap();

    let deleted = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            DeleteUser {
                id: created.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(deleted.last_state().id(), created.id());
    assert!(lab.users.rows.lock().unwrap().is_empty());
    assert_eq!(lab.policy.reloads(), 1);
    assert_eq!(
        permissions.permissions()[1],
        Permission::DeleteUser(created.id().clone())
    );
}

#[tokio::test]
async fn a_delete_of_a_missing_user_is_not_found_and_does_not_reload() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let err = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            DeleteUser {
                id: UserId::new("nobody"),
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::NotFound(_)));
    assert_eq!(lab.policy.reloads(), 0);
}

#[tokio::test]
async fn a_read_checks_its_permission() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create("seen").await.unwrap();

    let read = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            GetUser {
                id: created.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(read.id(), created.id());
    assert_eq!(
        permissions.permissions()[1],
        Permission::ReadUser(created.id().clone())
    );
}

#[tokio::test]
async fn a_read_by_username_asks_for_list_users() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create("admin").await.unwrap();

    let read = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            GetUserByUsername {
                username: Username::new("admin").unwrap(),
            },
        )
        .await
        .unwrap();

    assert_eq!(read.id(), created.id());
    assert_eq!(permissions.permissions()[1], Permission::ListUsers);
}

#[tokio::test]
async fn users_in_order_keeps_the_page_order_and_metadata_and_drops_unknown_ids() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    let bob = lab.create("bob").await.unwrap();
    let carol = lab.create("carol").await.unwrap();
    let ids = vec![bob.id().clone(), UserId::new("ghost"), carol.id().clone()];
    let page = PaginatedResult::new(ids, &PaginationParams::default(), 7);

    let users = users_in_order(lab.users.as_ref(), page).await.unwrap();

    let got: Vec<&UserId> = users.items().iter().map(User::id).collect();
    assert_eq!(got, vec![bob.id(), carol.id()]);
    assert_eq!(users.metadata().total_count(), 7);
}

fn held(user: &User, role: &str, scope: Scope) -> Grant {
    Grant::new(
        Principal::User(user.id().clone()),
        RoleName::new(role).unwrap(),
        scope,
    )
}

async fn delete_with(grants: impl Fn(&User, &User) -> Vec<Grant>) -> DomainResult<()> {
    let doomed = user("doomed");
    let other = user("other");
    let lab = lab_with(
        Arc::new(RecordingPermissionService::new()),
        grants(&doomed, &other),
    );
    lab.users
        .rows
        .lock()
        .unwrap()
        .insert(doomed.id().clone(), doomed.clone());
    lab.actions
        .run(
            &lab.uc,
            &alice(),
            DeleteUser {
                id: doomed.id().clone(),
            },
        )
        .await
        .map(|_| ())
}

fn o1() -> Scope {
    Scope::Organization(OrganizationId::new("o1"))
}

#[tokio::test]
async fn the_last_admin_of_an_organization_or_of_the_system_cannot_be_deleted() {
    for scope in [o1(), Scope::System] {
        let role = if scope == Scope::System {
            SYSTEM_ADMIN_ROLE
        } else {
            ORGANIZATION_ADMIN_ROLE
        };
        let err = delete_with(|doomed, _| vec![held(doomed, role, scope.clone())])
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::BusinessRule(_)), "{scope}");
    }
}

#[tokio::test]
async fn an_admin_is_deleted_when_another_human_admin_remains() {
    delete_with(|doomed, other| {
        vec![
            held(doomed, ORGANIZATION_ADMIN_ROLE, o1()),
            held(other, ORGANIZATION_ADMIN_ROLE, o1()),
        ]
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn the_last_project_admin_is_deleted() {
    delete_with(|doomed, _| {
        vec![held(
            doomed,
            PROJECT_ADMIN_ROLE,
            Scope::Project(ProjectId::new("p1")),
        )]
    })
    .await
    .unwrap();
}
