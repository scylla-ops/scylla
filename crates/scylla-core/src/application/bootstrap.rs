use crate::application::caller::{CallerContext, ServiceIdentity};
use crate::application::permission::service::PermissionService;
use crate::application::user::UserUseCases;
use crate::application::user_role::UserRoleUseCases;
use crate::application::{HashService, UserRepository, UserRoleRepository};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::role::name::RoleName;
use crate::domain::value_objects::user::{Email, Password, Username};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

/// Idempotent system bootstrap. Composes the user + role use cases to ensure a
/// privileged user exists and carries the given role on every boot. All
/// operations run as `CallerContext::Service(ServiceIdentity::bootstrap())`,
/// which the static `service` Cedar policy permits — so every step is gated by
/// the normal permission pipeline and audited like any other call.
#[derive(Constructor)]
pub struct BootstrapUseCases<
    U: UserRepository,
    H: HashService,
    PS: PermissionService,
    URR: UserRoleRepository,
> {
    user_uc: Arc<UserUseCases<U, H, PS>>,
    user_role_uc: Arc<UserRoleUseCases<URR, PS>>,
}

impl<U, H, PS, URR> BootstrapUseCases<U, H, PS, URR>
where
    U: UserRepository,
    H: HashService,
    PS: PermissionService,
    URR: UserRoleRepository,
{
    /// Ensure `username` exists (create if missing) and carries `role`. Returns
    /// `Ok(())` whether the user is freshly created or already present. Any
    /// other failure (validation, infrastructure, forbidden) bubbles up as
    /// [`DomainError`] for the caller to map at the layer boundary.
    #[instrument(skip(self, password))]
    pub async fn bootstrap_admin(
        &self,
        username: Username,
        email: Option<Email>,
        password: Password,
        role: RoleName,
    ) -> DomainResult<()> {
        let caller = CallerContext::Service(ServiceIdentity::bootstrap());

        let user = match self
            .user_uc
            .create(&caller, username.clone(), email, password)
            .await
        {
            Ok(user) => {
                tracing::info!(
                    user_id = %user.id(),
                    username = %username,
                    "bootstrap user created",
                );
                user
            }
            Err(DomainError::Conflict(_)) => {
                tracing::debug!(username = %username, "bootstrap user already exists");
                self.user_uc.get_by_username(&caller, &username).await?
            }
            Err(e) => return Err(e),
        };

        self.user_role_uc
            .assign(&caller, user.id(), &role)
            .await?;
        tracing::info!(
            user_id = %user.id(),
            role = %role,
            "bootstrap role ensured",
        );
        Ok(())
    }
}
