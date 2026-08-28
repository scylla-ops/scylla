use crate::application::HashService;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::app::credential_repository::AppCredentialRepository;
use crate::application::app::repository::AppRepository;
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::domain::app::{App, AppCredential};
use crate::domain::app::{AppName, AppSecret, AppSecretLabel};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppCredentialId, AppId, OrganizationId};
use crate::domain::permission::Permission;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

/// Label given to an App's first secret, created alongside the App itself.
const DEFAULT_SECRET_LABEL: &str = "default";

/// What a successful `create` returns: the persisted app plus its plaintext
/// secret, which is presented exactly once and never stored or retrievable.
pub struct CreatedApp {
    pub app: App,
    pub secret: AppSecret,
}

/// What a successful secret creation/regeneration returns: the persisted secret
/// record plus its plaintext, shown exactly once.
pub struct CreatedAppSecret {
    pub credential: AppCredential,
    pub secret: AppSecret,
}

/// Org-scoped management of machine Apps — generic credentials. Every method is
/// Cedar-gated, so an org-admin can manage the apps of orgs they control (admins
/// can manage any). An app is just an identity: it carries no authorization until
/// grants are assigned to it. A *agent* (an app that runs jobs) is provisioned
/// separately via `AgentUseCases`, which also grants it the agent role.
///
/// An App can hold several secrets; secret management (create/list/revoke/
/// enable) is gated on the app permissions: reading uses `ReadApp`, mutating
/// uses `DeleteApp` (the manage-level app permission).
#[derive(Constructor)]
pub struct AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService,
    PS: PermissionService,
    PC: PolicyControl,
{
    app_repo: Arc<A>,
    credential_repo: Arc<C>,
    hash_service: Arc<H>,
    permission_service: Arc<PS>,
    /// Live agent-stream registry. Disabling/deleting an app drops its stream
    /// here so a connected agent stops at once. No-op for apps that aren't
    /// connected agents.
    registry: Arc<dyn AgentDispatch>,
    policy_control: Arc<PC>,
}

impl<A, C, H, PS, PC> AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService,
    PS: PermissionService,
    PC: PolicyControl,
{
    #[instrument(skip_all, fields(org_id = %organization_id, name = %name))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        organization_id: OrganizationId,
        name: AppName,
    ) -> DomainResult<CreatedApp> {
        self.permission_service
            .check(caller, Permission::CreateApp(organization_id.clone()))
            .await?;

        let secret = crate::application::app::mint_app_secret();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let app = App::create(organization_id, name);
        let credential = AppCredential::create(
            app.id().clone(),
            AppSecretLabel::new(DEFAULT_SECRET_LABEL)?,
            secret_hash,
        );

        // A plain app is an identity only — no grant, no policy reload. It gains
        // capabilities later through explicit grants (or becomes an agent). The
        // initial secret is written in the same tx so it can authenticate.
        self.app_repo.create_app(&app, &credential).await?;

        Ok(CreatedApp { app, secret })
    }

    #[instrument(skip_all, fields(org_id = %organization_id))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        organization_id: OrganizationId,
    ) -> DomainResult<Vec<App>> {
        self.permission_service
            .check(
                caller,
                Permission::ListAppsByOrganization(organization_id.clone()),
            )
            .await?;
        self.app_repo.list_by_organization(&organization_id).await
    }

    #[instrument(skip_all, fields(app_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: AppId) -> DomainResult<App> {
        self.permission_service
            .check(caller, Permission::ReadApp(id.clone()))
            .await?;
        self.app_repo.find_by_id(&id).await
    }

    #[instrument(skip_all, fields(app_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: AppId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteApp(id.clone()))
            .await?;
        // A DB trigger drops every grant this app held with the row; reload so
        // the live policy set stops carrying their dead links.
        self.app_repo.delete(&id).await?;
        self.policy_control.reload().await
    }

    /// Enable or disable the whole app. Disabling has three effects: new token
    /// issuance is blocked and existing token lookups fail (the auth join filters
    /// on `is_active`); every privileged action is denied at the permission
    /// chokepoint, which re-checks `is_active` per request (so a live stream
    /// can't keep acting); and any open agent stream is dropped immediately.
    #[instrument(skip_all, fields(app_id = %id, active))]
    pub async fn set_active(
        &self,
        caller: &CallerContext,
        id: AppId,
        active: bool,
    ) -> DomainResult<App> {
        self.permission_service
            .check(caller, Permission::DeleteApp(id.clone()))
            .await?;
        self.app_repo.set_active(&id, active).await?;
        // Cut the live stream on disable so a connected agent stops at once
        // rather than running on its already-open connection. The per-action
        // liveness re-check in the permission service is the durable guarantee;
        // this just makes the effect instant and frees the registry slot.
        if !active {
            self.registry.disconnect(&id);
        }
        self.app_repo.find_by_id(&id).await
    }

    // --- Secret management -------------------------------------------------

    /// Create (regenerate) a new secret for an app. Returns the plaintext once.
    #[instrument(skip_all, fields(app_id = %app_id, label = %label))]
    pub async fn create_secret(
        &self,
        caller: &CallerContext,
        app_id: AppId,
        label: AppSecretLabel,
    ) -> DomainResult<CreatedAppSecret> {
        self.permission_service
            .check(caller, Permission::DeleteApp(app_id.clone()))
            .await?;
        // Ensure the app exists (surfaces NOT_FOUND rather than a dangling secret).
        self.app_repo.find_by_id(&app_id).await?;

        let secret = crate::application::app::mint_app_secret();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let credential = AppCredential::create(app_id, label, secret_hash);
        self.credential_repo.create(&credential).await?;

        Ok(CreatedAppSecret { credential, secret })
    }

    /// List an app's secrets (metadata only — never the plaintext).
    #[instrument(skip_all, fields(app_id = %app_id))]
    pub async fn list_secrets(
        &self,
        caller: &CallerContext,
        app_id: AppId,
    ) -> DomainResult<Vec<AppCredential>> {
        self.permission_service
            .check(caller, Permission::ReadApp(app_id.clone()))
            .await?;
        self.credential_repo.list_by_app(&app_id).await
    }

    /// Permanently remove a secret. Calls made with it stop authenticating, and
    /// any agent currently streaming under this app is dropped at once so it
    /// can't keep running on its already-open (token-authed) stream — it will
    /// fail to re-auth on reconnect.
    #[instrument(skip_all, fields(secret_id = %secret_id))]
    pub async fn revoke_secret(
        &self,
        caller: &CallerContext,
        secret_id: AppCredentialId,
    ) -> DomainResult<()> {
        let credential = self.credential_repo.find_by_id(&secret_id).await?;
        self.permission_service
            .check(caller, Permission::DeleteApp(credential.app_id().clone()))
            .await?;
        self.credential_repo.delete(&secret_id).await?;
        // Cut the live agent stream: it was authed once at open and isn't
        // re-checked per message, so without this it keeps running. The registry
        // is keyed by app id; a reconnect with a still-valid *other* secret
        // re-registers, so this is safe for multi-secret apps.
        self.registry.disconnect(credential.app_id());
        Ok(())
    }

    /// Enable or disable a secret. A disabled secret is kept but rejected at
    /// auth. Returns the updated record. Disabling also drops any live agent
    /// stream of this app (see `revoke_secret`).
    #[instrument(skip_all, fields(secret_id = %secret_id, enabled))]
    pub async fn set_secret_enabled(
        &self,
        caller: &CallerContext,
        secret_id: AppCredentialId,
        enabled: bool,
    ) -> DomainResult<AppCredential> {
        let credential = self.credential_repo.find_by_id(&secret_id).await?;
        self.permission_service
            .check(caller, Permission::DeleteApp(credential.app_id().clone()))
            .await?;
        self.credential_repo
            .set_enabled(&secret_id, enabled)
            .await?;
        if !enabled {
            self.registry.disconnect(credential.app_id());
        }
        self.credential_repo.find_by_id(&secret_id).await
    }
}
