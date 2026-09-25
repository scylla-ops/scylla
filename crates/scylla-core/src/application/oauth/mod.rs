pub mod commands;
pub mod provider;
pub mod queries;
pub mod repository;

pub use commands::{OAuthAccount, OAuthCallback, OAuthSignIn};
pub use provider::{OAuthProvider, OAuthUserInfo, PROVIDER_GITHUB};
pub use queries::GetAuthUrl;
pub use repository::OAuthIdentityRepository;

use crate::application::{HashService, SessionRepository, SignupRepository, UserRepository};
use crate::domain::ids::{OrganizationId, UserId};
use derive_more::Constructor;
use scylla_auth::authz::PolicyControl;
use std::sync::Arc;

pub struct OAuthOutcome {
    pub token: String,
    pub user_id: UserId,
    pub account: AccountOutcome,
}

pub enum AccountOutcome {
    New { organization_id: OrganizationId },
    Existing,
}

/// The OAuth flow's stage runners, one block per action in `commands.rs` and `queries.rs`.
/// Every action is `Public`: the provider's answer is the credential.
#[allow(clippy::too_many_arguments)]
#[derive(Constructor)]
pub struct OAuthUseCases {
    pub(super) provider: Arc<dyn OAuthProvider>,
    pub(super) identity_repo: Arc<dyn OAuthIdentityRepository>,
    pub(super) signup_repo: Arc<dyn SignupRepository>,
    pub(super) user_repo: Arc<dyn UserRepository>,
    pub(super) session_repo: Arc<dyn SessionRepository>,
    pub(super) hash_service: Arc<dyn HashService>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
}

#[cfg(test)]
mod tests;
