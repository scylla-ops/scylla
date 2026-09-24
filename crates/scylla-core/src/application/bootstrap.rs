use crate::application::GrantUseCases;
use crate::application::grant::CreateGrant;
use crate::application::user::{CreateUser, GetUserByUsername, UserUseCases};
use crate::domain::caller::{CallerContext, ServiceIdentity};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::role::RoleName;
use crate::domain::user::{Email, Password, Username};
use derive_more::Constructor;
use scylla_auth::authz::{Principal, Scope};
use scylla_extension::Actions;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct BootstrapUseCases {
    actions: Arc<Actions>,
    user_uc: Arc<UserUseCases>,
    grant_uc: Arc<GrantUseCases>,
}

impl BootstrapUseCases {
    #[instrument(skip_all, fields(username = %username.as_str(), role = %role.as_str()))]
    pub async fn bootstrap_admin(
        &self,
        username: Username,
        email: Option<Email>,
        password: Password,
        role: RoleName,
    ) -> DomainResult<()> {
        let caller = CallerContext::Service(ServiceIdentity::bootstrap());

        let create = CreateUser {
            username: username.clone(),
            email,
            password,
        };
        let user = match self.actions.run(&*self.user_uc, &caller, create).await {
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
                let get = GetUserByUsername {
                    username: username.clone(),
                };
                self.actions.run(&*self.user_uc, &caller, get).await?
            }
            Err(e) => return Err(e),
        };

        let grant = CreateGrant {
            principal: Principal::User(user.id().clone()),
            role: role.clone(),
            scope: Scope::System,
        };
        self.actions.run(&*self.grant_uc, &caller, grant).await?;
        tracing::info!(
            user_id = %user.id(),
            role = %role,
            "bootstrap system grant ensured",
        );
        Ok(())
    }
}
