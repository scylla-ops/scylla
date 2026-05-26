use crate::config::BootstrapConfig;
use crate::error::BootstrapError;
use scylla_core::application::{
    BootstrapUseCases, HashService, PermissionService, UserRepository, UserRoleRepository,
};
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::value_objects::role::name::RoleName;
use scylla_core::domain::value_objects::user::{Email, Password, Username};

/// Validate the on-disk bootstrap config into domain value objects and run the
/// [`BootstrapUseCases`]. Orchestration (create-or-fetch user, assign role)
/// lives in the use case; this shim is only the config → VO adapter at the API
/// boundary.
pub async fn bootstrap_admin<U, H, PS, URR>(
    bootstrap_uc: &BootstrapUseCases<U, H, PS, URR>,
    cfg: &BootstrapConfig,
) -> Result<(), BootstrapError>
where
    U: UserRepository,
    H: HashService,
    PS: PermissionService,
    URR: UserRoleRepository,
{
    let username = Username::new(&cfg.username).map_err(BootstrapError::InvalidUsername)?;
    let password = Password::new(&cfg.password).map_err(BootstrapError::InvalidPassword)?;
    let email = cfg
        .email
        .as_deref()
        .map(Email::new)
        .transpose()
        .map_err(BootstrapError::InvalidEmail)?;
    let role = RoleName::new("admin").map_err(BootstrapError::GrantPermission)?;

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
