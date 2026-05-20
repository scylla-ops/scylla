use crate::application::permission::grant::Grant;
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
}
