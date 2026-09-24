pub mod commands;
pub mod credential_repository;
pub mod queries;
pub mod repository;
pub mod secret_mint;
pub mod token_repository;
pub mod token_use_case;

pub use commands::{
    CreateApp, CreateAppSecret, CreatedApp, CreatedAppSecret, DeleteApp, NewApp, NewAppSecret,
    SetAppActive,
};
pub use credential_repository::AppCredentialRepository;
pub use queries::{GetApp, ListAppSecrets, ListApps};
pub use repository::AppRepository;
pub use secret_mint::mint_app_secret;
pub use token_repository::AppTokenRepository;
pub use token_use_case::{AppTokenOutcome, AppTokenUseCases};

use crate::application::HashService;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::domain::app::AppCredential;
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::ids::AppCredentialId;
use crate::domain::permission::Permission;
use derive_more::Constructor;
use scylla_auth::authz::{PermissionService, PolicyControl};
use std::sync::Arc;
use tracing::instrument;

/// The app aggregate's stage runners, one block per action in `commands.rs` and `queries.rs`.
/// `revoke_secret` and `set_secret_enabled` stay outside the pipeline: their permission is on
/// the secret's app, which only the loaded credential knows, and `Describe` sees the command
/// alone.
#[derive(Constructor)]
pub struct AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService,
    PS: PermissionService,
    PC: PolicyControl,
{
    pub(super) app_repo: Arc<A>,
    pub(super) credential_repo: Arc<C>,
    pub(super) hash_service: Arc<H>,
    pub(super) permission_service: Arc<PS>,
    pub(super) registry: Arc<dyn AgentDispatch>,
    pub(super) policy_control: Arc<PC>,
}

impl<A, C, H, PS, PC> AppUseCases<A, C, H, PS, PC>
where
    A: AppRepository,
    C: AppCredentialRepository,
    H: HashService,
    PS: PermissionService,
    PC: PolicyControl,
{
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

#[cfg(test)]
mod tests;
