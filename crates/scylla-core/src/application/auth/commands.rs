//! The session's writes. One block per command, in the order it runs: the struct, its access,
//! its payload types, what `Prepare` builds, what `Persist` writes.

use super::{AuthUseCases, new_session};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::session::Session;
use crate::domain::user::{Email, Password, Username};
use async_trait::async_trait;
use scylla_extension::{
    Access, Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};

/// No `Debug`: `password` is the credential. `identifier` is a username or an email.
pub struct Login {
    pub identifier: String,
    pub password: Password,
}

impl Describe for Login {
    fn access(&self) -> Access {
        Access::Public
    }
}

impl Command for Login {
    type Staged = Draft<Session>;
    type Committed = Session;
}

/// Same opaque error for both paths so callers cannot probe which accounts exist.
#[async_trait]
impl Run<Prepare<Login>> for AuthUseCases {
    async fn run(&self, input: Authorized<Login>) -> DomainResult<Prepared<Login>> {
        let cmd = input.command();
        let invalid = || DomainError::unauthorized("Invalid username or password");
        let lookup = if cmd.identifier.contains('@') {
            let email = Email::new(&cmd.identifier).map_err(|_| invalid())?;
            self.user_repo.find_by_email(&email).await
        } else {
            let username = Username::new(&cmd.identifier).map_err(|_| invalid())?;
            self.user_repo.find_by_username(&username).await
        };
        let user = lookup.map_err(|_| invalid())?;

        if !user.is_active() {
            return Err(DomainError::unauthorized("User account is inactive"));
        }
        if !self
            .hash_service
            .verify(&cmd.password, user.password_hash())
            .await?
        {
            return Err(invalid());
        }
        let session = new_session(user.id().clone());
        Ok(input.prepared(Draft::new(session)))
    }
}

#[async_trait]
impl Run<Persist<Login>> for AuthUseCases {
    async fn run(&self, input: Prepared<Login>) -> DomainResult<Committed<Login>> {
        input
            .commit(async |draft| self.session_repo.create(&draft.into_inner()).await)
            .await
    }
}

/// No `Debug`: `token` is the session credential.
pub struct RevokeToken {
    pub token: String,
}

impl Describe for RevokeToken {
    fn access(&self) -> Access {
        Access::Public
    }
}

impl Command for RevokeToken {
    type Staged = String;
    type Committed = ();
}

#[async_trait]
impl Run<Prepare<RevokeToken>> for AuthUseCases {
    async fn run(&self, input: Authorized<RevokeToken>) -> DomainResult<Prepared<RevokeToken>> {
        let token = input.command().token.clone();
        Ok(input.prepared(token))
    }
}

#[async_trait]
impl Run<Persist<RevokeToken>> for AuthUseCases {
    async fn run(&self, input: Prepared<RevokeToken>) -> DomainResult<Committed<RevokeToken>> {
        input
            .commit(async |token| self.session_repo.delete_by_token(&token).await)
            .await
    }
}
