//! The signup's writes. One block per command, in the order it runs: the struct, its access,
//! its payload types, what `Prepare` builds, what `Persist` writes.

use super::{NewAccount, SignupOutcome, SignupUseCases};
use crate::application::auth::new_session;
use crate::domain::errors::DomainResult;
use crate::domain::organization::OrganizationName;
use crate::domain::session::Session;
use crate::domain::user::{Email, Password, User, Username};
use async_trait::async_trait;
use scylla_extension::{
    Access, Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};

/// `Prepare` hashes `password`, so only the hash is staged.
#[derive(Debug)]
pub struct Signup {
    pub username: Username,
    pub email: Email,
    pub password: Password,
    pub organization_name: OrganizationName,
}

pub struct NewSignup {
    pub account: NewAccount,
    pub session: Session,
}

impl Describe for Signup {
    fn access(&self) -> Access {
        Access::Public
    }
}

impl Command for Signup {
    type Staged = Draft<NewSignup>;
    type Committed = SignupOutcome;
}

#[async_trait]
impl Run<Prepare<Signup>> for SignupUseCases {
    async fn run(&self, input: Authorized<Signup>) -> DomainResult<Prepared<Signup>> {
        let cmd = input.command();
        let password_hash = self.hash_service.hash(&cmd.password).await?;
        let user = User::create(cmd.username.clone(), Some(cmd.email.clone()), password_hash);
        let account = NewAccount::new(user, cmd.organization_name.clone())?;
        let session = new_session(account.user.id().clone());
        Ok(input.prepared(Draft::new(NewSignup { account, session })))
    }
}

#[async_trait]
impl Run<Persist<Signup>> for SignupUseCases {
    async fn run(&self, input: Prepared<Signup>) -> DomainResult<Committed<Signup>> {
        input
            .commit(async |draft| {
                let NewSignup { account, session } = draft.into_inner();
                self.signup_repo
                    .provision_account(&account.user, &account.organization, &account.grant)
                    .await?;
                self.policy_control.reload().await?;
                self.session_repo.create(&session).await?;
                Ok(SignupOutcome {
                    token: session.token().to_string(),
                    user_id: account.user.id().clone(),
                    organization_id: account.organization.id().clone(),
                })
            })
            .await
    }
}
