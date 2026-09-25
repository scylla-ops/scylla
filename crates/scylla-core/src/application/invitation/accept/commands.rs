//! The invitation accept's writes. One block per command, in the order it runs: the struct, its
//! access, its payload types, what `Prepare` builds, what `Persist` writes.

use super::{AcceptOutcome, InvitationAcceptUseCases};
use crate::application::auth::new_session;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::invitation::Invitation;
use crate::domain::role::RoleName;
use crate::domain::session::Session;
use crate::domain::user::{Password, User, Username};
use async_trait::async_trait;
use scylla_auth::authz::{Grant, ORGANIZATION_MEMBER_ROLE, Principal, Scope};
use scylla_extension::{
    Access, Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};

/// No `Debug`: `token` and `password` are credentials. `Prepare` hashes the password, so only
/// the hash is staged.
pub struct AcceptInvitation {
    pub token: String,
    pub username: Username,
    pub password: Password,
}

/// `new_user` is `None` when the invited email already has an account: the grant joins it.
pub struct Acceptance {
    pub invitation: Invitation,
    pub new_user: Option<User>,
    pub grant: Grant,
    pub session: Session,
}

impl Describe for AcceptInvitation {
    fn access(&self) -> Access {
        Access::Public
    }
}

impl Command for AcceptInvitation {
    type Staged = Draft<Acceptance>;
    type Committed = AcceptOutcome;
}

#[async_trait]
impl Run<Prepare<AcceptInvitation>> for InvitationAcceptUseCases {
    async fn run(
        &self,
        input: Authorized<AcceptInvitation>,
    ) -> DomainResult<Prepared<AcceptInvitation>> {
        let cmd = input.command();
        let invitation = self.invite_repo.find_by_token(&cmd.token).await?;
        if !invitation.is_acceptable() {
            return Err(DomainError::business_rule("Invitation is no longer valid"));
        }

        let (new_user, user_id) =
            if let Ok(existing) = self.user_repo.find_by_email(invitation.email()).await {
                (None, existing.id().clone())
            } else {
                let password_hash = self.hash_service.hash(&cmd.password).await?;
                let user = User::create(
                    cmd.username.clone(),
                    Some(invitation.email().clone()),
                    password_hash,
                );
                let id = user.id().clone();
                (Some(user), id)
            };

        // The grant is the join: a roleless invite still mints `organization-member`.
        let role = match invitation.role() {
            Some(role) => role.clone(),
            None => RoleName::new(ORGANIZATION_MEMBER_ROLE)?,
        };
        let grant = Grant::new(
            Principal::User(user_id.clone()),
            role,
            Scope::Organization(invitation.organization_id().clone()),
        );
        Ok(input.prepared(Draft::new(Acceptance {
            invitation,
            new_user,
            grant,
            session: new_session(user_id),
        })))
    }
}

#[async_trait]
impl Run<Persist<AcceptInvitation>> for InvitationAcceptUseCases {
    async fn run(
        &self,
        input: Prepared<AcceptInvitation>,
    ) -> DomainResult<Committed<AcceptInvitation>> {
        input
            .commit(async |draft| {
                let Acceptance {
                    invitation,
                    new_user,
                    grant,
                    session,
                } = draft.into_inner();
                self.invite_repo
                    .accept_atomic(
                        invitation.id(),
                        new_user.as_ref(),
                        session.user_id(),
                        &grant,
                    )
                    .await?;
                self.policy_control.reload().await?;
                self.session_repo.create(&session).await?;
                Ok(AcceptOutcome {
                    token: session.token().to_string(),
                    user_id: session.user_id().clone(),
                    organization_id: invitation.organization_id().clone(),
                })
            })
            .await
    }
}
