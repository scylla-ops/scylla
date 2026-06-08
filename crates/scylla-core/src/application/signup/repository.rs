use crate::application::authz::grant::Grant;
use crate::domain::entities::{Organization, User};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Atomic provisioning of a brand-new account. The four inserts (user,
/// organization, membership, organization-admin grant) must succeed or fail as a
/// unit — a partial signup would leave an orphan user or an unowned org. The
/// Postgres adapter runs them in a single transaction; a username unique
/// violation rolls everything back and surfaces as [`DomainError::Conflict`].
///
/// [`DomainError::Conflict`]: crate::domain::errors::DomainError::Conflict
#[async_trait]
pub trait SignupRepository: Send + Sync {
    async fn provision_account(
        &self,
        user: &User,
        organization: &Organization,
        grant: &Grant,
    ) -> DomainResult<()>;

    /// Like [`provision_account`], but also links an external OAuth identity in
    /// the SAME transaction. An OAuth first-login can then never half-commit: the
    /// account, its org, the owner grant and the identity link all succeed or all
    /// roll back. Without this, a failure after the account commits leaves an
    /// account with no linked identity — unrecoverable for a GitHub account with
    /// no email (the email-relink fallback can't fire).
    ///
    /// [`provision_account`]: SignupRepository::provision_account
    #[cfg(feature = "oauth-github")]
    async fn provision_account_with_identity(
        &self,
        user: &User,
        organization: &Organization,
        grant: &Grant,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<()>;
}
