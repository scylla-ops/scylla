use crate::domain::errors::DomainResult;
use crate::domain::organization::Organization;
use crate::domain::user::User;
use async_trait::async_trait;
use scylla_auth::authz::Grant;

#[async_trait]
pub trait SignupRepository: Send + Sync {
    async fn provision_account(
        &self,
        user: &User,
        organization: &Organization,
        grant: &Grant,
    ) -> DomainResult<()>;

    /// Same transaction as the account: an account with no linked identity and no email would be unrecoverable.
    async fn provision_account_with_identity(
        &self,
        user: &User,
        organization: &Organization,
        grant: &Grant,
        provider: &str,
        provider_user_id: &str,
    ) -> DomainResult<()>;
}
