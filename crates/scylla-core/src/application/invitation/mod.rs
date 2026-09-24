pub mod accept;
pub mod commands;
pub mod queries;
pub mod repository;
pub mod token;

pub use accept::{AcceptOutcome, InvitationAcceptUseCases};
pub use commands::{CreateInvitation, NewInvitation};
pub use queries::ListInvitations;
pub use repository::InvitationRepository;
pub use token::mint_invitation_token;

use crate::application::OrganizationRepository;
use crate::application::mail::Mailer;
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::ids::InvitationId;
use crate::domain::permission::Permission;
use derive_more::Constructor;
use scylla_auth::authz::{PermissionService, RoleRepository};
use std::sync::Arc;
use tracing::instrument;

/// The invitation aggregate's stage runners, one block per action in `commands.rs` and
/// `queries.rs`. `revoke` stays outside the pipeline: its permission is on the invitation's
/// organization, which only the loaded invitation knows, and `Describe` sees the command alone.
/// Accepting runs before the invitee has an account, so it lives in `InvitationAcceptUseCases`.
#[derive(Constructor)]
pub struct InvitationUseCases {
    pub(super) invite_repo: Arc<dyn InvitationRepository>,
    pub(super) org_repo: Arc<dyn OrganizationRepository>,
    pub(super) role_repo: Arc<dyn RoleRepository>,
    pub(super) mailer: Arc<dyn Mailer>,
    pub(super) permission_service: Arc<dyn PermissionService>,
}

impl InvitationUseCases {
    #[instrument(skip_all, fields(invite_id = %invite_id))]
    pub async fn revoke(
        &self,
        caller: &CallerContext,
        invite_id: &InvitationId,
    ) -> DomainResult<()> {
        let invite = self.invite_repo.find_by_id(invite_id).await?;
        self.permission_service
            .check(
                caller,
                Permission::ManageInvitations(invite.organization_id().clone()),
            )
            .await?;
        self.invite_repo.revoke(invite_id).await
    }
}

#[cfg(test)]
mod tests;
