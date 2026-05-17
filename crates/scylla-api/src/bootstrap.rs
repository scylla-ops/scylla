use crate::config::BootstrapConfig;
use anyhow::{Context, Result};
use scylla_core::application::UserUseCases;
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::value_objects::permission::policy::Policy;
use scylla_core::domain::value_objects::user::{Password, Username};

pub async fn bootstrap_admin<
    U: scylla_core::application::ports::UserRepository,
    H: scylla_core::application::ports::HashService,
    P: scylla_core::application::ports::PermissionService,
>(
    user_uc: &UserUseCases<U, H>,
    permission_service: &mut P,
    bootstrap: &BootstrapConfig,
) -> Result<()> {
    let username = Username::new(&bootstrap.username).context("Invalid bootstrap username")?;
    let password = Password::new(&bootstrap.password).context("Invalid bootstrap password")?;

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
                .context("Failed to add admin permissions for bootstrap user")?;

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
        Err(e) => return Err(e).context("Failed to bootstrap admin user"),
    }

    Ok(())
}
