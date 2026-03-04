use derive_more::Constructor;
use domain::entities::EntityId;
use domain::errors::DomainResult;
use domain::ports::services::permission_service::PermissionService;
use domain::value_objects::permission::policy::{GroupingPolicy, Policy};
use std::sync::Arc;

#[derive(Constructor)]
pub struct PermissionUseCases<PS: PermissionService> {
    permission_service: Arc<PS>,
}

impl<PS: PermissionService> PermissionUseCases<PS> {
    pub async fn add_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
        self.permission_service.add_policy(sub, policy).await
    }

    pub async fn remove_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
        self.permission_service.remove_policy(sub, policy).await
    }

    pub async fn list_policies(
        &self,
        subject: Option<&str>,
    ) -> DomainResult<Vec<(String, Policy)>> {
        self.permission_service.list_policies(subject).await
    }

    pub async fn add_grouping_policy(
        &self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool> {
        self.permission_service
            .add_grouping_policy(sub, policy)
            .await
    }

    pub async fn remove_grouping_policy(
        &self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool> {
        self.permission_service
            .remove_grouping_policy(sub, policy)
            .await
    }

    pub async fn list_grouping_policies(
        &self,
        subject: Option<&str>,
    ) -> DomainResult<Vec<(String, GroupingPolicy)>> {
        self.permission_service
            .list_grouping_policies(subject)
            .await
    }
}
