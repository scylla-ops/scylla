//! The invitation's writes. One block per command, in the order it runs: the struct, its
//! access, its payload types, what `Prepare` builds, what `Persist` writes.

use super::InvitationUseCases;
use crate::application::actions::user_only;
use crate::application::invitation::token::mint_invitation_token;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{InvitationId, OrganizationId};
use crate::domain::invitation::{Invitation, InvitationStatus};
use crate::domain::organization::OrganizationName;
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use crate::domain::user::Email;
use async_trait::async_trait;
use scylla_auth::authz::{ORGANIZATION_MEMBER_ROLE, PrincipalKind, Scope, check_grantable};
use scylla_extension::{
    Access, Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};

#[derive(Debug)]
pub struct CreateInvitation {
    pub organization_id: OrganizationId,
    pub email: Email,
    pub role: Option<RoleName>,
}

/// No `Debug`: `token` is the credential. The store keeps its digest; the mail carries it with
/// the organization's name.
pub struct NewInvitation {
    pub invitation: Invitation,
    pub token: String,
    pub organization_name: OrganizationName,
}

impl Describe for CreateInvitation {
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageInvitations(self.organization_id.clone()))
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
        let invited_by = user_only(input.caller())?;
        let role = cmd
            .role
            .clone()
            .map_or_else(|| RoleName::new(ORGANIZATION_MEMBER_ROLE), Ok)?;
        check_grantable(
            &self.role_repo.list_all().await?,
            &self.grant_repo.list_all().await?,
            input.caller(),
            PrincipalKind::User,
            &role,
            &Scope::Organization(cmd.organization_id.clone()),
            Some(&cmd.organization_id),
        )?;
        let org = self.org_repo.find_by_id(&cmd.organization_id).await?;
        let invitation = Invitation::create(
            cmd.organization_id.clone(),
            cmd.email.clone(),
            cmd.role.clone(),
            invited_by,
        );
        Ok(input.prepared(Draft::new(NewInvitation {
            invitation,
            token: mint_invitation_token(),
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
                    token,
                    organization_name,
                } = draft.into_inner();
                self.invite_repo.create(&invitation, &token).await?;
                let body = format!(
                    "<p>You've been invited to join <b>{}</b> on Scylla.</p>\
                     <p>Use this token to accept: <code>{}</code></p>",
                    organization_name.as_str(),
                    token
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
    fn access(&self) -> Access {
        Access::Requires(Permission::RevokeInvitation(self.id.clone()))
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
        if invitation.status() != InvitationStatus::Pending {
            return Err(DomainError::business_rule(
                "Only a pending invitation can be revoked",
            ));
        }
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
