//! The invitation's writes. One block per command, in the order it runs: the struct, its
//! permission, its payload types, what `Prepare` builds, what `Persist` writes.

use super::InvitationUseCases;
use crate::application::invitation::token::mint_invitation_token;
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{InvitationId, OrganizationId, UserId};
use crate::domain::invitation::Invitation;
use crate::domain::organization::OrganizationName;
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use crate::domain::user::Email;
use async_trait::async_trait;
use scylla_auth::authz::{Scope, validate_role_in_db};
use scylla_extension::{
    Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};

#[derive(Debug)]
pub struct CreateInvitation {
    pub organization_id: OrganizationId,
    pub email: Email,
    pub role: Option<RoleName>,
}

/// The organization's name goes into the mail sent once the invitation is stored.
#[derive(Debug)]
pub struct NewInvitation {
    pub invitation: Invitation,
    pub organization_name: OrganizationName,
}

impl Describe for CreateInvitation {
    fn permission(&self) -> Permission {
        Permission::ManageInvitations(self.organization_id.clone())
    }
}

impl Command for CreateInvitation {
    type Staged = Draft<NewInvitation>;
    type Committed = Invitation;
}

#[async_trait]
impl Run<Prepare<CreateInvitation>> for InvitationUseCases {
    async fn run(
        &self,
        input: Authorized<CreateInvitation>,
    ) -> DomainResult<Prepared<CreateInvitation>> {
        let cmd = input.command();
        if let Some(role) = &cmd.role {
            validate_role_in_db(
                &*self.role_repo,
                role,
                &Scope::Organization(cmd.organization_id.clone()),
            )
            .await?;
        }
        let org = self.org_repo.find_by_id(&cmd.organization_id).await?;
        let invited_by = match input.caller() {
            CallerContext::User(id) => id.clone(),
            _ => UserId::new("system"),
        };
        let invitation = Invitation::create(
            cmd.organization_id.clone(),
            cmd.email.clone(),
            cmd.role.clone(),
            invited_by,
            mint_invitation_token(),
        );
        Ok(input.prepared(Draft::new(NewInvitation {
            invitation,
            organization_name: org.name().clone(),
        })))
    }
}

#[async_trait]
impl Run<Persist<CreateInvitation>> for InvitationUseCases {
    async fn run(
        &self,
        input: Prepared<CreateInvitation>,
    ) -> DomainResult<Committed<CreateInvitation>> {
        input
            .commit(async |draft| {
                let NewInvitation {
                    invitation,
                    organization_name,
                } = draft.into_inner();
                self.invite_repo.create(&invitation).await?;
                let body = format!(
                    "<p>You've been invited to join <b>{}</b> on Scylla.</p>\
                     <p>Use this token to accept: <code>{}</code></p>",
                    organization_name.as_str(),
                    invitation.token()
                );
                // Best-effort: a transient SMTP failure must not lose the persisted invitation.
                if let Err(e) = self
                    .mailer
                    .send(invitation.email(), "You've been invited to Scylla", &body)
                    .await
                {
                    tracing::warn!(error = %e, invite_id = %invitation.id(), "invite email send failed");
                }
                Ok(invitation)
            })
            .await
    }
}

#[derive(Debug)]
pub struct RevokeInvitation {
    pub id: InvitationId,
}

impl Describe for RevokeInvitation {
    fn permission(&self) -> Permission {
        Permission::RevokeInvitation(self.id.clone())
    }
}

impl Command for RevokeInvitation {
    type Staged = Invitation;
    type Committed = Invitation;
}

#[async_trait]
impl Run<Prepare<RevokeInvitation>> for InvitationUseCases {
    async fn run(
        &self,
        input: Authorized<RevokeInvitation>,
    ) -> DomainResult<Prepared<RevokeInvitation>> {
        let invitation = self.invite_repo.find_by_id(&input.command().id).await?;
        Ok(input.prepared(invitation))
    }
}

#[async_trait]
impl Run<Persist<RevokeInvitation>> for InvitationUseCases {
    async fn run(
        &self,
        input: Prepared<RevokeInvitation>,
    ) -> DomainResult<Committed<RevokeInvitation>> {
        input
            .commit(async |invitation| {
                self.invite_repo.revoke(invitation.id()).await?;
                Ok(invitation.revoked())
            })
            .await
    }
}
