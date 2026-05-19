use crate::config::BootstrapConfig;
use crate::error::BootstrapError;
use scylla_core::application::{HashService, PermissionService, UserRepository, UserUseCases};
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::value_objects::permission::policy::Policy;
use scylla_core::domain::value_objects::user::{Password, Username};

pub async fn bootstrap_admin<U, H, P>(
    user_uc: &UserUseCases<U, H>,
    permission_service: &mut P,
    bootstrap: &BootstrapConfig,
) -> Result<(), BootstrapError>
where
    U: UserRepository,
    H: HashService,
    P: PermissionService,
{
    let username = Username::new(&bootstrap.username).map_err(BootstrapError::InvalidUsername)?;
    let password = Password::new(&bootstrap.password).map_err(BootstrapError::InvalidPassword)?;

    match user_uc.create(username, password).await {
        Ok(user) => {
            tracing::info!(
                "Bootstrap user '{}' created (id: {})",
                bootstrap.username,
                user.id()
            );

            permission_service
                .add_policy(user.id().clone(), Policy::absolute())
                .await
                .map_err(BootstrapError::GrantPermission)?;

            tracing::info!(
                "Admin permissions granted to bootstrap user '{}'",
                bootstrap.username
            );
        }
        Err(DomainError::Conflict(_)) => {
            tracing::debug!(
                "Bootstrap user '{}' already exists, skipping",
                bootstrap.username
            );
        }
        Err(e) => return Err(BootstrapError::CreateUser(e)),
    }

    Ok(())
}
