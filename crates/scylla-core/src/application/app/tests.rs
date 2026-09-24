//! The app's actions through the engine, on stub ports.

use super::*;
use crate::application::PermissionAuthorizer;
use crate::application::agent::JobDispatch;
use crate::domain::agent::Agent;
use crate::domain::app::{App, AppName, AppSecret, AppSecretHash, AppSecretLabel};
use crate::domain::errors::DomainError;
use crate::domain::ids::{AppId, OrganizationId, UserId};
use crate::domain::user::{Password, PasswordHash};
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService};
use async_trait::async_trait;
use scylla_auth::authz::Grant;
use scylla_extension::{Actions, Hooks};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubApps {
    rows: Mutex<HashMap<AppId, App>>,
    credentials: Mutex<Vec<AppCredential>>,
}

#[async_trait]
impl AppRepository for StubApps {
    async fn create_app(&self, app: &App, credential: &AppCredential) -> DomainResult<()> {
        self.rows
            .lock()
            .unwrap()
            .insert(app.id().clone(), app.clone());
        self.credentials.lock().unwrap().push(credential.clone());
        Ok(())
    }
    async fn provision_agent(
        &self,
        _: &App,
        _: &AppCredential,
        _: &Agent,
        _: &Grant,
    ) -> DomainResult<()> {
        unreachable!("no agent in an app action")
    }
    async fn provision(&self, _: &App, _: &AppCredential, _: &Grant) -> DomainResult<()> {
        unreachable!("no grant in an app action")
    }
    async fn find_by_id(&self, id: &AppId) -> DomainResult<App> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("App", id.to_string()))
    }
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
    ) -> DomainResult<Vec<App>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .values()
            .filter(|a| a.organization_id() == organization_id)
            .cloned()
            .collect())
    }
    async fn set_active(&self, id: &AppId, active: bool) -> DomainResult<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(a) = rows.get(id).cloned() {
            rows.insert(
                id.clone(),
                App::from_persistence(
                    a.id().clone(),
                    a.organization_id().clone(),
                    a.name().clone(),
                    active,
                    a.created_at(),
                    a.updated_at(),
                ),
            );
        }
        Ok(())
    }
    async fn delete(&self, id: &AppId) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(id);
        Ok(())
    }
}

#[derive(Default)]
struct StubCredentials {
    rows: Mutex<HashMap<AppCredentialId, AppCredential>>,
}

#[async_trait]
impl AppCredentialRepository for StubCredentials {
    async fn create(&self, credential: &AppCredential) -> DomainResult<()> {
        self.rows
            .lock()
            .unwrap()
            .insert(credential.id().clone(), credential.clone());
        Ok(())
    }
    async fn find_by_id(&self, id: &AppCredentialId) -> DomainResult<AppCredential> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("AppCredential", id.to_string()))
    }
    async fn list_by_app(&self, app_id: &AppId) -> DomainResult<Vec<AppCredential>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.app_id() == app_id)
            .cloned()
            .collect())
    }
    async fn list_enabled_by_app(&self, _: &AppId) -> DomainResult<Vec<AppCredential>> {
        unreachable!("only the token exchange lists enabled secrets")
    }
    async fn set_enabled(&self, id: &AppCredentialId, enabled: bool) -> DomainResult<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(c) = rows.get(id).cloned() {
            rows.insert(
                id.clone(),
                AppCredential::from_persistence(
                    c.id().clone(),
                    c.app_id().clone(),
                    c.label().clone(),
                    c.secret_hash().clone(),
                    enabled,
                    c.created_at(),
                    c.updated_at(),
                ),
            );
        }
        Ok(())
    }
    async fn delete(&self, id: &AppCredentialId) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(id);
        Ok(())
    }
}

#[derive(Default)]
struct StubHash {
    hashed: Mutex<usize>,
}

#[async_trait]
impl HashService for StubHash {
    async fn hash(&self, _: &Password) -> DomainResult<PasswordHash> {
        unreachable!("no password in an app action")
    }
    async fn verify(&self, _: &Password, _: &PasswordHash) -> DomainResult<bool> {
        unreachable!("no password in an app action")
    }
    async fn hash_secret(&self, _: &AppSecret) -> DomainResult<AppSecretHash> {
        *self.hashed.lock().unwrap() += 1;
        AppSecretHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def")
    }
    async fn verify_secret(&self, _: &AppSecret, _: &AppSecretHash) -> DomainResult<bool> {
        unreachable!("no secret check in an app action")
    }
}

#[derive(Default)]
struct StubRegistry {
    disconnected: Mutex<Vec<AppId>>,
}

#[async_trait]
impl AgentDispatch for StubRegistry {
    fn connected(&self) -> Vec<AppId> {
        Vec::new()
    }
    async fn dispatch(&self, _: &AppId, _: &JobDispatch) -> DomainResult<()> {
        unreachable!("no dispatch in an app action")
    }
    fn disconnect(&self, app_id: &AppId) {
        self.disconnected.lock().unwrap().push(app_id.clone());
    }
    fn in_flight(&self, _: &AppId) -> usize {
        0
    }
    fn release(&self, _: &AppId) {}
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
    uc: AppUseCases,
    apps: Arc<StubApps>,
    credentials: Arc<StubCredentials>,
    hash: Arc<StubHash>,
    registry: Arc<StubRegistry>,
    policy: Arc<StubPolicy>,
}

impl Lab {
    async fn create(&self) -> DomainResult<CreatedApp> {
        self.actions.run(&self.uc, &alice(), create()).await
    }

    async fn create_secret(&self, app_id: &AppId) -> DomainResult<CreatedAppSecret> {
        self.actions
            .run(
                &self.uc,
                &alice(),
                CreateAppSecret {
                    app_id: app_id.clone(),
                    label: AppSecretLabel::new("ci").unwrap(),
                },
            )
            .await
    }
}

fn lab(permissions: Arc<dyn PermissionService>) -> Lab {
    let apps = Arc::new(StubApps::default());
    let credentials = Arc::new(StubCredentials::default());
    let hash = Arc::new(StubHash::default());
    let registry = Arc::new(StubRegistry::default());
    let policy = Arc::new(StubPolicy::default());
    Lab {
        actions: Actions::new(
            Arc::new(PermissionAuthorizer::new(permissions.clone())),
            Arc::new(Hooks::new()),
        ),
        uc: AppUseCases::new(
            apps.clone(),
            credentials.clone(),
            hash.clone(),
            permissions,
            registry.clone(),
            policy.clone(),
        ),
        apps,
        credentials,
        hash,
        registry,
        policy,
    }
}

fn alice() -> CallerContext {
    CallerContext::User(UserId::new("alice"))
}

fn org() -> OrganizationId {
    OrganizationId::new("acme")
}

fn create() -> CreateApp {
    CreateApp {
        organization_id: org(),
        name: AppName::new("ci-bot").unwrap(),
    }
}

#[tokio::test]
async fn a_create_checks_the_permission_then_stores_the_app_with_its_default_secret() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());

    let created = lab.create().await.unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::CreateApp(org())]
    );
    assert!(!created.secret.as_str().is_empty());
    assert!(lab.apps.rows.lock().unwrap().contains_key(created.app.id()));
    let credentials = lab.apps.credentials.lock().unwrap();
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].app_id(), created.app.id());
    assert_eq!(credentials[0].label().as_str(), "default");
}

#[tokio::test]
async fn a_denied_create_never_hashes_or_persists() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));

    let err = lab.create().await.err().unwrap();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert_eq!(*lab.hash.hashed.lock().unwrap(), 0);
    assert!(lab.apps.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn reads_check_their_permission() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create().await.unwrap();
    let id = created.app.id().clone();

    let app = lab
        .actions
        .run(&lab.uc, &alice(), GetApp { id: id.clone() })
        .await
        .unwrap();
    let apps = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            ListApps {
                organization_id: org(),
            },
        )
        .await
        .unwrap();
    let secrets = lab
        .actions
        .run(&lab.uc, &alice(), ListAppSecrets { app_id: id.clone() })
        .await
        .unwrap();

    assert_eq!(app.id(), &id);
    assert_eq!(apps.len(), 1);
    assert!(secrets.is_empty());
    assert_eq!(
        permissions.permissions()[1..],
        [
            Permission::ReadApp(id.clone()),
            Permission::ListAppsByOrganization(org()),
            Permission::ReadApp(id),
        ]
    );
}

#[tokio::test]
async fn a_deactivation_disconnects_the_app_and_returns_the_fresh_row() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create().await.unwrap();
    let id = created.app.id().clone();

    let app = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            SetAppActive {
                id: id.clone(),
                is_active: false,
            },
        )
        .await
        .unwrap();

    assert!(!app.is_active());
    assert_eq!(*lab.registry.disconnected.lock().unwrap(), vec![id.clone()]);
    assert_eq!(permissions.permissions()[1], Permission::DeleteApp(id));
}

#[tokio::test]
async fn an_activation_keeps_the_stream() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    let created = lab.create().await.unwrap();

    lab.actions
        .run(
            &lab.uc,
            &alice(),
            SetAppActive {
                id: created.app.id().clone(),
                is_active: true,
            },
        )
        .await
        .unwrap();

    assert!(lab.registry.disconnected.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_delete_removes_the_app_and_reloads_the_policies() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create().await.unwrap();
    let id = created.app.id().clone();

    let deleted = lab
        .actions
        .run(&lab.uc, &alice(), DeleteApp { id: id.clone() })
        .await
        .unwrap();

    assert_eq!(deleted.last_state(), &id);
    assert!(lab.apps.rows.lock().unwrap().is_empty());
    assert_eq!(*lab.policy.reloads.lock().unwrap(), 1);
    assert_eq!(permissions.permissions()[1], Permission::DeleteApp(id));
}

#[tokio::test]
async fn a_secret_create_checks_the_app_exists_then_stores_the_credential() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create().await.unwrap();
    let id = created.app.id().clone();

    let secret = lab.create_secret(&id).await.unwrap();

    assert_eq!(secret.credential.app_id(), &id);
    assert_eq!(secret.credential.label().as_str(), "ci");
    assert!(
        lab.credentials
            .rows
            .lock()
            .unwrap()
            .contains_key(secret.credential.id())
    );
    assert_eq!(permissions.permissions()[1], Permission::DeleteApp(id));
}

#[tokio::test]
async fn a_secret_for_a_missing_app_is_not_found_and_never_hashed() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let err = lab.create_secret(&AppId::new("ghost")).await.err().unwrap();

    assert!(matches!(err, DomainError::NotFound { .. }));
    assert_eq!(*lab.hash.hashed.lock().unwrap(), 0);
    assert!(lab.credentials.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_revoke_checks_the_permission_on_the_loaded_secret_app_and_disconnects_it() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create().await.unwrap();
    let id = created.app.id().clone();
    let secret = lab.create_secret(&id).await.unwrap();

    lab.uc
        .revoke_secret(&alice(), secret.credential.id().clone())
        .await
        .unwrap();

    assert!(lab.credentials.rows.lock().unwrap().is_empty());
    assert_eq!(*lab.registry.disconnected.lock().unwrap(), vec![id.clone()]);
    assert_eq!(permissions.permissions()[2], Permission::DeleteApp(id));
}

#[tokio::test]
async fn disabling_a_secret_disconnects_its_app_and_returns_the_fresh_row() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create().await.unwrap();
    let id = created.app.id().clone();
    let secret = lab.create_secret(&id).await.unwrap();

    let credential = lab
        .uc
        .set_secret_enabled(&alice(), secret.credential.id().clone(), false)
        .await
        .unwrap();

    assert!(!credential.is_enabled());
    assert_eq!(*lab.registry.disconnected.lock().unwrap(), vec![id.clone()]);
    assert_eq!(permissions.permissions()[2], Permission::DeleteApp(id));
}
