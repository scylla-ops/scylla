use crate::application::ports::services::permission_service::PermissionService;
use crate::domain::entities::EntityId;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::policy::{GroupingPolicy, Policy};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct PermissionUseCases<PS: PermissionService> {
    permission_service: Arc<PS>,
}

impl<PS: PermissionService> PermissionUseCases<PS> {
    #[instrument(skip(self, sub))]
    pub async fn add_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
        self.permission_service.add_policy(sub, policy).await
    }

    #[instrument(skip(self, sub))]
    pub async fn remove_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
        self.permission_service.remove_policy(sub, policy).await
    }

    #[instrument(skip(self))]
    pub async fn list_policies(
        &self,
        subject: Option<&str>,
    ) -> DomainResult<Vec<(String, Policy)>> {
        self.permission_service.list_policies(subject).await
    }

    #[instrument(skip(self, sub))]
    pub async fn add_grouping_policy(
        &self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool> {
        self.permission_service
            .add_grouping_policy(sub, policy)
            .await
    }

    #[instrument(skip(self, sub))]
    pub async fn remove_grouping_policy(
        &self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool> {
        self.permission_service
            .remove_grouping_policy(sub, policy)
            .await
    }

    #[instrument(skip(self))]
    pub async fn list_grouping_policies(
        &self,
        subject: Option<&str>,
    ) -> DomainResult<Vec<(String, GroupingPolicy)>> {
        self.permission_service
            .list_grouping_policies(subject)
            .await
    }
}

