use crate::application::authz::grant::SYSTEM_ADMIN_ROLE;
use crate::application::authz::policy::PolicyControl;
use crate::application::{
    BootstrapUseCases, GrantRepository, HashService, PermissionService, UserRepository,
};
use crate::config::BootstrapConfig;
use crate::error::BootstrapError;
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::role::RoleName;
use scylla_core::domain::user::{Email, Password, Username};

/// Validate the on-disk bootstrap config into domain value objects and run the
/// [`BootstrapUseCases`]. Orchestration (create-or-fetch user, assign role)
/// lives in the use case; this shim is only the config → VO adapter at the API
/// boundary.
pub async fn bootstrap_admin<U, H, PS, G, PC>(
    bootstrap_uc: &BootstrapUseCases<U, H, PS, G, PC>,
    cfg: &BootstrapConfig,
) -> Result<(), BootstrapError>
where
    U: UserRepository,
    H: HashService,
    PS: PermissionService,
    G: GrantRepository,
    PC: PolicyControl,
{
    // Loudly flag the well-known dev default so it can't silently ship to a real
    // deployment. The credential is `BootstrapConfig::default()` (admin/admin123).
    if cfg.username == "admin" && cfg.password == "admin123" {
        tracing::warn!(
            "Bootstrapping the admin account with the DEFAULT credentials (admin/admin123). \
             Change `bootstrap.password` before exposing this instance — these are public."
        );
    }

    let username = Username::new(&cfg.username).map_err(BootstrapError::InvalidUsername)?;
    let password = Password::new(&cfg.password).map_err(BootstrapError::InvalidPassword)?;
    let email = cfg
        .email
        .as_deref()
        .map(Email::new)
        .transpose()
        .map_err(BootstrapError::InvalidEmail)?;
    let role = RoleName::new(SYSTEM_ADMIN_ROLE).map_err(BootstrapError::GrantPermission)?;

    bootstrap_uc
        .bootstrap_admin(username, email, password, role)
        .await
        .map_err(|e| match e {
            // Permission service refused — service policy or role-step denial.
            DomainError::Forbidden(_) => BootstrapError::GrantPermission(e),
            // Everything else (validation, conflict resolution, infrastructure)
            // happened around the user-creation / fetch flow.
            _ => BootstrapError::CreateUser(e),
        })
}
