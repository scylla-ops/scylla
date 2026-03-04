use crate::config::BootstrapConfig;
use anyhow::{Context, Result};
use application::UserUseCases;
use domain::errors::DomainError;
use domain::value_objects::permission::policy::Policy;
use domain::value_objects::user::{Password, UserName};

pub async fn bootstrap_admin<
    U: domain::ports::UserRepository,
    H: domain::ports::HashService,
    P: domain::ports::PermissionService,
>(
    user_uc: &UserUseCases<U, H>,
    permission_service: &mut P,
    bootstrap: &BootstrapConfig,
) -> Result<()> {
    let username = UserName::new(&bootstrap.username).context("Invalid bootstrap username")?;
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
