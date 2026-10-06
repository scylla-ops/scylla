pub mod commands;

pub use commands::{AcceptInvitation, Acceptance};

use crate::application::invitation::InvitationRepository;
use crate::application::{HashService, SessionRepository, UserRepository};
use crate::domain::ids::{OrganizationId, UserId};
use derive_more::Constructor;
use std::sync::Arc;

pub struct AcceptOutcome {
    pub token: String,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
}

/// The invitation accept's stage runners, one block per action in `commands.rs`.
/// `AcceptInvitation` is `Public`: the invitee has no account yet, so the token is the credential.
#[derive(Constructor)]
pub struct InvitationAcceptUseCases {
    pub(super) invite_repo: Arc<dyn InvitationRepository>,
    pub(super) user_repo: Arc<dyn UserRepository>,
    pub(super) hash_service: Arc<dyn HashService>,
    pub(super) session_repo: Arc<dyn SessionRepository>,
}

#[cfg(test)]
mod tests;
