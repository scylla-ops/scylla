use crate::application::HashService;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::application::app::credential_repository::AppCredentialRepository;
use crate::application::app::repository::AppRepository;
use crate::domain::app::{App, AppCredential};
use crate::domain::app::{AppName, AppSecret, AppSecretLabel};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppCredentialId, AppId, OrganizationId};
use crate::domain::permission::Permission;
use derive_more::Constructor;
use scylla_auth::authz::{PermissionService, PolicyControl};
use scylla_auth::caller::CallerContext;
use std::sync::Arc;
use tracing::instrument;

const DEFAULT_SECRET_LABEL: &str = "default";

pub struct CreatedApp {
    pub app: App,
    pub secret: AppSecret,
}

pub struct CreatedAppSecret {
    pub credential: AppCredential,
    pub secret: AppSecret,
}

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
        // A DB trigger drops the app's grants with the row; reload so the live set stops carrying them.
        self.app_repo.delete(&id).await?;
        self.policy_control.reload().await
    }

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
        // The per-action liveness check is the durable guarantee; this makes the effect instant.
        if !active {
            self.registry.disconnect(&id);
        }
        self.app_repo.find_by_id(&id).await
    }

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
        self.app_repo.find_by_id(&app_id).await?;

        let secret = crate::application::app::mint_app_secret();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let credential = AppCredential::create(app_id, label, secret_hash);
        self.credential_repo.create(&credential).await?;

        Ok(CreatedAppSecret { credential, secret })
    }

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
        // The stream was authenticated once at open; a reconnect with another enabled secret re-registers.
        self.registry.disconnect(credential.app_id());
        Ok(())
    }

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
