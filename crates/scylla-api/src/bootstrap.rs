use crate::config::BootstrapConfig;
use crate::error::BootstrapError;
use scylla_core::application::caller::{CallerContext, ServiceIdentity};
use scylla_core::application::{
    HashService, PermissionService, UserRepository, UserRoleRepository, UserUseCases,
};
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::value_objects::role::name::RoleName;
use scylla_core::domain::value_objects::user::{Password, Username};

/// Idempotent first-boot bootstrap: create the `admin` user if missing, then
/// ensure it carries the `admin` role (`Scylla::Role::"admin"`). Cedar's
/// admin policy grants this role full access; granting/revoking the role is
/// the only mutable knob, since policy text itself is static.
pub async fn bootstrap_admin<U, H, PS, URR>(
    user_uc: &UserUseCases<U, H, PS>,
    user_role_repo: &URR,
    bootstrap: &BootstrapConfig,
) -> Result<(), BootstrapError>
where
    U: UserRepository,
    H: HashService,
    PS: PermissionService,
    URR: UserRoleRepository,
{
    let username = Username::new(&bootstrap.username).map_err(BootstrapError::InvalidUsername)?;
    let password = Password::new(&bootstrap.password).map_err(BootstrapError::InvalidPassword)?;
    let role = RoleName::new("admin").map_err(BootstrapError::GrantPermission)?;
    let caller = CallerContext::Service(ServiceIdentity::bootstrap());

    let user = match user_uc.create(&caller, username, password).await {
        Ok(user) => {
            tracing::info!(
                "Bootstrap user '{}' created (id: {})",
                bootstrap.username,
                user.id()
            );
            user
        }
        Err(DomainError::Conflict(_)) => {
            tracing::debug!(
                "Bootstrap user '{}' already exists, skipping create",
                bootstrap.username
            );
            let existing_username = Username::new(&bootstrap.username)
                .map_err(BootstrapError::InvalidUsername)?;
            user_uc
                .get_by_username(&caller, &existing_username)
                .await
                .map_err(BootstrapError::CreateUser)?
        }
        Err(e) => return Err(BootstrapError::CreateUser(e)),
    };

    user_role_repo
        .assign(user.id(), &role)
        .await
        .map_err(BootstrapError::GrantPermission)?;
    tracing::info!(
        "Admin role ensured for bootstrap user '{}'",
        bootstrap.username
    );

    Ok(())
}
