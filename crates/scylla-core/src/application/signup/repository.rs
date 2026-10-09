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
        grants: &[Grant],
    ) -> DomainResult<()>;
}
