use crate::domain::entities::EntityId;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::policy::{GroupingPolicy, Policy};

#[async_trait::async_trait]
pub trait PermissionService: Send + Sync {
    async fn check(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool>;

    async fn add_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool>;

    async fn remove_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool>;

    async fn list_policies(&self, subject: Option<&str>) -> DomainResult<Vec<(String, Policy)>>;

    async fn add_grouping_policy(
        &self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool>;

    async fn remove_grouping_policy(
        &self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool>;

    async fn list_grouping_policies(
        &self,
        subject: Option<&str>,
    ) -> DomainResult<Vec<(String, GroupingPolicy)>>;
}
