use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, SessionId, UserId};
use crate::domain::organization::OrganizationName;
use crate::domain::project::ProjectName;
use crate::domain::role::{RoleDisplayName, RoleName};
use crate::domain::user::{PasswordReset, ResetToken, User};
use async_trait::async_trait;
use chrono::Duration;
use scylla_auth::authz::Scope;

/// One grant of a user, with the names to show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAccess {
    pub grant_id: String,
    pub scope: Scope,
    /// The organization of an organization or a project scope.
    pub organization_id: Option<OrganizationId>,
    pub organization_name: Option<OrganizationName>,
    pub project_name: Option<ProjectName>,
    pub role: RoleName,
    pub role_name: RoleDisplayName,
}

/// What a user signs in with, beyond its row: its sessions, its reset links, and its grants with
/// their names. A write that also writes the user follows the version rule of
/// `UserRepository::update`, and does all its work in one transaction.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Writes the user, deletes each session of the user except `keep`, and deletes each reset
    /// link of the user.
    async fn update_signed_out(&self, user: &User, keep: Option<&SessionId>) -> DomainResult<User>;

    /// Deletes each session of the user except `keep`, and returns the count.
    async fn revoke_sessions(
        &self,
        user_id: &UserId,
        keep: Option<&SessionId>,
    ) -> DomainResult<u64>;

    /// Stores the link and deletes the other links of the user. With a `cooldown`, it stores
    /// nothing and returns `false` when the user has a link made within it.
    async fn issue_reset(
        &self,
        reset: &PasswordReset,
        cooldown: Option<Duration>,
    ) -> DomainResult<bool>;

    /// `NotFound` for a token that no link has.
    async fn find_reset(&self, token: &ResetToken) -> DomainResult<PasswordReset>;

    /// Marks the link used, writes the user, deletes the other links and each session of the
    /// user. A link that is used or expired at the write gives `reset_link_invalid`.
    async fn redeem_reset(&self, reset: &PasswordReset, user: &User) -> DomainResult<User>;

    /// The system grants first, then by organization name, then by project name.
    async fn list_access(&self, user_id: &UserId) -> DomainResult<Vec<UserAccess>>;
}
