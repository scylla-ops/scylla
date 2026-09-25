//! The user's writes. One block per command, in the order it runs: the struct, its
//! access, its payload types, what `Prepare` builds, what `Persist` writes.

use super::UserUseCases;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::UserId;
use crate::domain::permission::Permission;
use crate::domain::user::{Email, Password, User, Username};
use async_trait::async_trait;
use scylla_extension::{
    Access, Authorized, Command, Committed, Deleted, Describe, Draft, Persist, Prepare, Prepared,
    Run,
};

#[derive(Debug)]
pub struct CreateUser {
    pub username: Username,
    pub email: Option<Email>,
    pub password: Password,
}

impl Describe for CreateUser {
    fn access(&self) -> Access {
        Access::Requires(Permission::CreateUser)
    }
}

impl Command for CreateUser {
    type Staged = Draft<User>;
    type Committed = User;
}

#[async_trait]
impl Run<Prepare<CreateUser>> for UserUseCases {
    async fn run(&self, input: Authorized<CreateUser>) -> DomainResult<Prepared<CreateUser>> {
        let cmd = input.command();
        if self.user_repo.username_exists(&cmd.username).await? {
            return Err(DomainError::conflict("Username already exists"));
        }
        let password_hash = self.hash_service.hash(&cmd.password).await?;
        let user = User::create(cmd.username.clone(), cmd.email.clone(), password_hash);
        Ok(input.prepared(Draft::new(user)))
    }
}

#[async_trait]
impl Run<Persist<CreateUser>> for UserUseCases {
    async fn run(&self, input: Prepared<CreateUser>) -> DomainResult<Committed<CreateUser>> {
        input
            .commit(async |draft| self.user_repo.create(&draft.into_inner()).await)
            .await
    }
}

#[derive(Debug)]
pub struct UpdateUser {
    pub id: UserId,
    pub username: Option<Username>,
}

impl Describe for UpdateUser {
    fn access(&self) -> Access {
        Access::Requires(Permission::UpdateUser(self.id.clone()))
    }
}

impl Command for UpdateUser {
    type Staged = Draft<User>;
    type Committed = User;
}

#[async_trait]
impl Run<Prepare<UpdateUser>> for UserUseCases {
    async fn run(&self, input: Authorized<UpdateUser>) -> DomainResult<Prepared<UpdateUser>> {
        let cmd = input.command();
        let mut user = self.user_repo.find_by_id(&cmd.id).await?;
        if let Some(username) = &cmd.username {
            if self.user_repo.username_exists(username).await? && user.username() != username {
                return Err(DomainError::conflict("Username already exists"));
            }
            user.update_username(username.clone())?;
        }
        Ok(input.prepared(Draft::new(user)))
    }
}

#[async_trait]
impl Run<Persist<UpdateUser>> for UserUseCases {
    async fn run(&self, input: Prepared<UpdateUser>) -> DomainResult<Committed<UpdateUser>> {
        input
            .commit(async |draft| self.user_repo.update(&draft.into_inner()).await)
            .await
    }
}

#[derive(Debug)]
pub struct DeleteUser {
    pub id: UserId,
}

impl Describe for DeleteUser {
    fn access(&self) -> Access {
        Access::Requires(Permission::DeleteUser(self.id.clone()))
    }
}

impl Command for DeleteUser {
    type Staged = User;
    type Committed = Deleted<User>;
}

#[async_trait]
impl Run<Prepare<DeleteUser>> for UserUseCases {
    async fn run(&self, input: Authorized<DeleteUser>) -> DomainResult<Prepared<DeleteUser>> {
        let user = self.user_repo.find_by_id(&input.command().id).await?;
        Ok(input.prepared(user))
    }
}

#[async_trait]
impl Run<Persist<DeleteUser>> for UserUseCases {
    // A DB trigger drops the user's grants with the row; the reload stops the live set carrying them.
    async fn run(&self, input: Prepared<DeleteUser>) -> DomainResult<Committed<DeleteUser>> {
        input
            .commit(async |user| {
                self.user_repo.delete(user.id()).await?;
                self.policy_control.reload().await?;
                Ok(Deleted::new(user))
            })
            .await
    }
}
