use crate::entities::EntityId;
use crate::errors::DomainResult;
use crate::value_objects::permission::policy::{GroupingPolicy, Policy};

#[async_trait::async_trait]
pub trait PermissionService: Send + Sync {
    async fn check(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool>;
    async fn add_policy(&mut self, sub: impl EntityId, policy: Policy) -> DomainResult<bool>;
    async fn add_grouping_policy(
        &mut self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool>;
}
