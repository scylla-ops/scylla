pub mod accept;
pub mod commands;
pub mod queries;
pub mod repository;
pub mod token;

pub use accept::{AcceptOutcome, InvitationAcceptUseCases};
pub use commands::{CreateInvitation, NewInvitation, RevokeInvitation};
pub use queries::ListInvitations;
pub use repository::InvitationRepository;
pub use token::mint_invitation_token;

use crate::application::OrganizationRepository;
use crate::application::mail::Mailer;
use derive_more::Constructor;
use scylla_auth::authz::RoleRepository;
use std::sync::Arc;

/// The invitation aggregate's stage runners, one block per action in `commands.rs` and
/// `queries.rs`. It has no method of its own; `Actions::run` drives it. Accepting runs before
/// the invitee has an account, so it lives in `InvitationAcceptUseCases`.
#[derive(Constructor)]
pub struct InvitationUseCases {
    pub(super) invite_repo: Arc<dyn InvitationRepository>,
    pub(super) org_repo: Arc<dyn OrganizationRepository>,
    pub(super) role_repo: Arc<dyn RoleRepository>,
    pub(super) mailer: Arc<dyn Mailer>,
}

#[cfg(test)]
mod tests;
