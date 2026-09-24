//! The user's actions through the engine, on stub ports.

use super::*;
use crate::application::PermissionAuthorizer;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::app::{AppSecret, AppSecretHash};
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::UserId;
use crate::domain::permission::Permission;
use crate::domain::user::{Email, Password, PasswordHash, User, Username};
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService};
use async_trait::async_trait;
use scylla_auth::authz::PermissionService;
use scylla_extension::{Actions, Hooks};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubUsers {
    rows: Mutex<HashMap<UserId, User>>,
}

#[async_trait]
impl UserRepository for StubUsers {
    async fn create(&self, user: &User) -> DomainResult<User> {
        self.rows
            .lock()
            .unwrap()
            .insert(user.id().clone(), user.clone());
        Ok(user.clone())
    }
    async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("User", id.to_string()))
    }
    async fn find_by_ids(&self, _: &[UserId]) -> DomainResult<Vec<User>> {
        Ok(Vec::new())
    }
    async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
        self.rows
            .lock()
            .unwrap()
            .values()
            .find(|u| u.username() == username)
            .cloned()
            .ok_or_else(|| DomainError::not_found("User", username.to_string()))
    }
    async fn find_by_email(&self, email: &Email) -> DomainResult<User> {
        Err(DomainError::not_found("User", email.to_string()))
    }
    async fn update(&self, user: &User) -> DomainResult<User> {
        self.create(user).await
    }
    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(id);
        Ok(())
    }
    async fn list_all(&self, _: Option<&PaginationParams>) -> DomainResult<PaginatedResult<User>> {
        Ok(PaginatedResult::new(
            Vec::new(),
            &PaginationParams::default(),
            0,
        ))
    }
    async fn username_exists(&self, username: &Username) -> DomainResult<bool> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .values()
            .any(|u| u.username() == username))
    }
}

struct StubHash;

#[async_trait]
impl HashService for StubHash {
    async fn hash(&self, _: &Password) -> DomainResult<PasswordHash> {
        PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def")
    }
    async fn verify(&self, _: &Password, _: &PasswordHash) -> DomainResult<bool> {
        unreachable!("no verify in a user action")
    }
    async fn hash_secret(&self, _: &AppSecret) -> DomainResult<AppSecretHash> {
        unreachable!("no secret in a user action")
    }
    async fn verify_secret(&self, _: &AppSecret, _: &AppSecretHash) -> DomainResult<bool> {
        unreachable!("no secret in a user action")
    }
}

#[derive(Default)]
struct StubPolicy {
    reloads: Mutex<usize>,
}

#[async_trait]
impl PolicyControl for StubPolicy {
    async fn reload(&self) -> DomainResult<()> {
        *self.reloads.lock().unwrap() += 1;
        Ok(())
    }
}

struct Lab {
    actions: Actions,
    uc: UserUseCases,
    users: Arc<StubUsers>,
    policy: Arc<StubPolicy>,
}

impl Lab {
    async fn create(&self, username: &str) -> DomainResult<User> {
        self.actions.run(&self.uc, &alice(), create(username)).await
    }
}

fn lab(permissions: Arc<dyn PermissionService>) -> Lab {
    let users = Arc::new(StubUsers::default());
    let policy = Arc::new(StubPolicy::default());
    Lab {
        actions: Actions::new(
            Arc::new(PermissionAuthorizer::new(permissions)),
            Arc::new(Hooks::new()),
        ),
        uc: UserUseCases::new(users.clone(), Arc::new(StubHash), policy.clone()),
        users,
        policy,
    }
}

fn alice() -> CallerContext {
    CallerContext::User(UserId::new("alice"))
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
    assert_eq!(*lab.policy.reloads.lock().unwrap(), 0);
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
    assert_eq!(*lab.policy.reloads.lock().unwrap(), 1);
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

    assert!(matches!(err, DomainError::NotFound { .. }));
    assert_eq!(*lab.policy.reloads.lock().unwrap(), 0);
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
