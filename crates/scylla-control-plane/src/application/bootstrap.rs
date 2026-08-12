use crate::application::authz::grant::{Grant, GrantUseCases, Principal, Scope};
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::service::PermissionService;
use crate::application::caller::{CallerContext, ServiceIdentity};
use crate::application::user::UserUseCases;
use crate::application::{GrantRepository, HashService, UserRepository};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::role::RoleName;
use crate::domain::user::{Email, Password, Username};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

/// Idempotent system bootstrap. Composes the user + grant use cases to ensure a
/// privileged user exists and holds a `system-admin` grant on the System scope
/// (the unified replacement for the former global role) on every boot. All
/// operations run as `CallerContext::Service(ServiceIdentity::bootstrap())`,
/// which the static `service` Cedar policy permits — so every step is gated by
/// the normal permission pipeline and audited like any other call.
#[derive(Constructor)]
pub struct BootstrapUseCases<
    U: UserRepository,
    H: HashService,
    PS: PermissionService,
    G: GrantRepository,
    PC: PolicyControl,
> {
    user_uc: Arc<UserUseCases<U, H, PS, PC>>,
    grant_uc: Arc<GrantUseCases<G, PC, PS>>,
}

impl<U, H, PS, G, PC> BootstrapUseCases<U, H, PS, G, PC>
where
    U: UserRepository,
    H: HashService,
    PS: PermissionService,
    G: GrantRepository,
    PC: PolicyControl,
{
    /// Ensure `username` exists (create if missing) and holds `role` on the
    /// System scope. Returns `Ok(())` whether the user is freshly created or
    /// already present (the grant insert is idempotent). Any other failure
    /// (validation, infrastructure, forbidden) bubbles up as [`DomainError`] for
    /// the caller to map at the layer boundary.
    #[instrument(skip_all, fields(username = %username.as_str(), role = %role.as_str()))]
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

        // Global authority is a System-scoped grant (idempotent insert + live
        // policy reload). Replaces the former `user_roles` role assignment.
        let grant = Grant::new(
            Principal::User(user.id().clone()),
            role.clone(),
            Scope::System,
        );
        self.grant_uc.grant(&caller, &grant).await?;
        tracing::info!(
            user_id = %user.id(),
            role = %role,
            "bootstrap system grant ensured",
        );
        Ok(())
    }
}
