//! The app's writes. One block per command, in the order it runs: the struct, its permission,
//! its payload types, what `Prepare` builds, what `Persist` writes. `DeleteApp` and
//! `SetAppActive` stage the id alone: the row is not read before the write, as before.

use super::AppUseCases;
use super::mint_app_secret;
use crate::application::{AppCredentialRepository, AppRepository, HashService};
use crate::domain::app::{App, AppCredential, AppName, AppSecret, AppSecretLabel};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, OrganizationId};
use crate::domain::permission::Permission;
use async_trait::async_trait;
use scylla_auth::authz::{PermissionService, PolicyControl};
use scylla_extension::{
    Authorized, Command, Committed, Deleted, Describe, Draft, Persist, Prepare, Prepared, Run,
};

const DEFAULT_SECRET_LABEL: &str = "default";

#[derive(Debug)]
pub struct CreateApp {
    pub organization_id: OrganizationId,
    pub name: AppName,
}

/// No `Debug`: `secret` is the plaintext the caller sees once.
pub struct NewApp {
    pub app: App,
    pub credential: AppCredential,
    pub secret: AppSecret,
}

pub struct CreatedApp {
    pub app: App,
    pub secret: AppSecret,
}

impl Describe for CreateApp {
    fn permission(&self) -> Permission {
        Permission::CreateApp(self.organization_id.clone())
    }
}

impl Command for CreateApp {
    type Staged = Draft<NewApp>;
    type Committed = CreatedApp;
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Prepare<CreateApp>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<CreateApp>) -> DomainResult<Prepared<CreateApp>> {
        let cmd = input.command();
        let secret = mint_app_secret();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let app = App::create(cmd.organization_id.clone(), cmd.name.clone());
        let credential = AppCredential::create(
            app.id().clone(),
            AppSecretLabel::new(DEFAULT_SECRET_LABEL)?,
            secret_hash,
        );
        Ok(input.prepared(Draft::new(NewApp {
            app,
            credential,
            secret,
        })))
    }
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Persist<CreateApp>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Prepared<CreateApp>) -> DomainResult<Committed<CreateApp>> {
        input
            .commit(async |draft| {
                let NewApp {
                    app,
                    credential,
                    secret,
                } = draft.into_inner();
                self.app_repo.create_app(&app, &credential).await?;
                Ok(CreatedApp { app, secret })
            })
            .await
    }
}

#[derive(Debug)]
pub struct SetAppActive {
    pub id: AppId,
    pub is_active: bool,
}

impl Describe for SetAppActive {
    fn permission(&self) -> Permission {
        Permission::DeleteApp(self.id.clone())
    }
}

impl Command for SetAppActive {
    type Staged = AppId;
    type Committed = App;
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Prepare<SetAppActive>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<SetAppActive>) -> DomainResult<Prepared<SetAppActive>> {
        let id = input.command().id.clone();
        Ok(input.prepared(id))
    }
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Persist<SetAppActive>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Prepared<SetAppActive>) -> DomainResult<Committed<SetAppActive>> {
        let active = input.command().is_active;
        input
            .commit(async |id| {
                self.app_repo.set_active(&id, active).await?;
                // The per-action liveness check is the durable guarantee; this makes the effect instant.
                if !active {
                    self.registry.disconnect(&id);
                }
                self.app_repo.find_by_id(&id).await
            })
            .await
    }
}

#[derive(Debug)]
pub struct DeleteApp {
    pub id: AppId,
}

impl Describe for DeleteApp {
    fn permission(&self) -> Permission {
        Permission::DeleteApp(self.id.clone())
    }
}

impl Command for DeleteApp {
    type Staged = AppId;
    type Committed = Deleted<AppId>;
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Prepare<DeleteApp>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<DeleteApp>) -> DomainResult<Prepared<DeleteApp>> {
        let id = input.command().id.clone();
        Ok(input.prepared(id))
    }
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Persist<DeleteApp>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Prepared<DeleteApp>) -> DomainResult<Committed<DeleteApp>> {
        input
            .commit(async |id| {
                // A DB trigger drops the app's grants with the row; reload so the live set stops carrying them.
                self.app_repo.delete(&id).await?;
                self.policy_control.reload().await?;
                Ok(Deleted::new(id))
            })
            .await
    }
}

#[derive(Debug)]
pub struct CreateAppSecret {
    pub app_id: AppId,
    pub label: AppSecretLabel,
}

/// No `Debug`: `secret` is the plaintext the caller sees once.
pub struct NewAppSecret {
    pub credential: AppCredential,
    pub secret: AppSecret,
}

pub struct CreatedAppSecret {
    pub credential: AppCredential,
    pub secret: AppSecret,
}

impl Describe for CreateAppSecret {
    fn permission(&self) -> Permission {
        Permission::DeleteApp(self.app_id.clone())
    }
}

impl Command for CreateAppSecret {
    type Staged = Draft<NewAppSecret>;
    type Committed = CreatedAppSecret;
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Prepare<CreateAppSecret>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(
        &self,
        input: Authorized<CreateAppSecret>,
    ) -> DomainResult<Prepared<CreateAppSecret>> {
        let cmd = input.command();
        self.app_repo.find_by_id(&cmd.app_id).await?;
        let secret = mint_app_secret();
        let secret_hash = self.hash_service.hash_secret(&secret).await?;
        let credential = AppCredential::create(cmd.app_id.clone(), cmd.label.clone(), secret_hash);
        Ok(input.prepared(Draft::new(NewAppSecret { credential, secret })))
    }
}

#[async_trait]
impl<A, C, H, PS, PC> Run<Persist<CreateAppSecret>> for AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(
        &self,
        input: Prepared<CreateAppSecret>,
    ) -> DomainResult<Committed<CreateAppSecret>> {
        input
            .commit(async |draft| {
                let NewAppSecret { credential, secret } = draft.into_inner();
                self.credential_repo.create(&credential).await?;
                Ok(CreatedAppSecret { credential, secret })
            })
            .await
    }
}
