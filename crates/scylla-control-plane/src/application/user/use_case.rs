use crate::application::authz::policy::PolicyControl;
use crate::application::caller::CallerContext;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::application::{HashService, PermissionService, UserRepository};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::UserId;
use crate::domain::permission::Permission;
use crate::domain::user::User;
use crate::domain::user::{Email, Password, Username};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct UserUseCases<U: UserRepository, H: HashService, PS: PermissionService, PC: PolicyControl>
{
    user_repo: Arc<U>,
    hash_service: Arc<H>,
    permission_service: Arc<PS>,
    policy_control: Arc<PC>,
}

impl<U: UserRepository, H: HashService, PS: PermissionService, PC: PolicyControl>
    UserUseCases<U, H, PS, PC>
{
    #[instrument(skip_all, fields(username = %username))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        username: Username,
        email: Option<Email>,
        password: Password,
    ) -> DomainResult<User> {
        self.permission_service
            .check(caller, Permission::CreateUser)
            .await?;

        if self.user_repo.username_exists(&username).await? {
            return Err(DomainError::conflict("Username already exists"));
        }

        let password_hash = self.hash_service.hash(&password).await?;
        let user = User::create(username, email, password_hash);
        self.user_repo.create(&user).await
    }

    #[instrument(skip_all, fields(user_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &UserId) -> DomainResult<User> {
        self.permission_service
            .check(caller, Permission::ReadUser(id.clone()))
            .await?;
        self.user_repo.find_by_id(id).await
    }

    /// Look up a user by their username, gated by the all-users read policy.
    /// Bootstrap uses this on the conflict branch to recover the existing
    /// admin user's id when the username already exists.
    #[instrument(skip_all, fields(username = %username))]
    pub async fn get_by_username(
        &self,
        caller: &CallerContext,
        username: &Username,
    ) -> DomainResult<User> {
        self.permission_service
            .check(caller, Permission::ListUsers)
            .await?;
        self.user_repo.find_by_username(username).await
    }

    #[instrument(skip_all, fields(user_id = %id))]
    pub async fn update(
        &self,
        caller: &CallerContext,
        id: &UserId,
        username: Option<Username>,
    ) -> DomainResult<User> {
        self.permission_service
            .check(caller, Permission::UpdateUser(id.clone()))
            .await?;

        let mut user = self.user_repo.find_by_id(id).await?;

        if let Some(new_username) = username {
            if self.user_repo.username_exists(&new_username).await?
                && user.username() != &new_username
            {
                return Err(DomainError::conflict("Username already exists"));
            }
            user.update_username(new_username)?;
        }

        self.user_repo.update(&user).await
    }

    #[instrument(skip_all, fields(user_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &UserId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteUser(id.clone()))
            .await?;
        self.user_repo.find_by_id(id).await?;
        // A DB trigger drops every grant this user held, at any scope, with the
        // row; reload so the live policy set stops carrying their dead links.
        self.user_repo.delete(id).await?;
        self.policy_control.reload().await
    }

    #[instrument(skip(self, caller, pagination))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>> {
        self.permission_service
            .check(caller, Permission::ListUsers)
            .await?;
        self.user_repo.list_all(pagination).await
    }
}
