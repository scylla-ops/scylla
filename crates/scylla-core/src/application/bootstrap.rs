use crate::application::GrantUseCases;
use crate::application::user::UserUseCases;
use crate::application::{HashService, UserRepository};
use crate::domain::caller::{CallerContext, ServiceIdentity};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::role::RoleName;
use crate::domain::user::{Email, Password, Username};
use derive_more::Constructor;
use scylla_auth::authz::{
    Grant, GrantRepository, PermissionService, PolicyControl, Principal, Scope,
};
use std::sync::Arc;
use tracing::instrument;

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
