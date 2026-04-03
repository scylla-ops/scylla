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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::services::permission_service::PermissionService;
    use crate::domain::entities::UserId;
    use crate::domain::value_objects::permission::{Act, Resource, Scope, Target};
    use crate::domain::value_objects::role::name::RoleName;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct StubPermissionService {
        add_policy_fn: Option<Box<dyn Fn(&str, &Policy) -> DomainResult<bool> + Send + Sync>>,
        remove_policy_fn: Option<Box<dyn Fn(&str, &Policy) -> DomainResult<bool> + Send + Sync>>,
        list_policies_fn: Option<Box<dyn Fn(Option<&str>) -> DomainResult<Vec<(String, Policy)>> + Send + Sync>>,
        add_grouping_fn: Option<Box<dyn Fn(&str, &GroupingPolicy) -> DomainResult<bool> + Send + Sync>>,
        remove_grouping_fn: Option<Box<dyn Fn(&str, &GroupingPolicy) -> DomainResult<bool> + Send + Sync>>,
        list_grouping_fn: Option<Box<dyn Fn(Option<&str>) -> DomainResult<Vec<(String, GroupingPolicy)>> + Send + Sync>>,
    }

    impl Default for StubPermissionService {
        fn default() -> Self {
            Self {
                add_policy_fn: None,
                remove_policy_fn: None,
                list_policies_fn: None,
                add_grouping_fn: None,
                remove_grouping_fn: None,
                list_grouping_fn: None,
            }
        }
    }

    #[async_trait]
    impl PermissionService for StubPermissionService {
        async fn check(&self, _sub: impl EntityId, _policy: Policy) -> DomainResult<bool> {
            unimplemented!()
        }
        async fn add_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
            (self.add_policy_fn.as_ref().unwrap())(sub.as_ref(), &policy)
        }
        async fn remove_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
            (self.remove_policy_fn.as_ref().unwrap())(sub.as_ref(), &policy)
        }
        async fn list_policies(&self, subject: Option<&str>) -> DomainResult<Vec<(String, Policy)>> {
            (self.list_policies_fn.as_ref().unwrap())(subject)
        }
        async fn add_grouping_policy(&self, sub: impl EntityId, policy: GroupingPolicy) -> DomainResult<bool> {
            (self.add_grouping_fn.as_ref().unwrap())(sub.as_ref(), &policy)
        }
        async fn remove_grouping_policy(&self, sub: impl EntityId, policy: GroupingPolicy) -> DomainResult<bool> {
            (self.remove_grouping_fn.as_ref().unwrap())(sub.as_ref(), &policy)
        }
        async fn list_grouping_policies(&self, subject: Option<&str>) -> DomainResult<Vec<(String, GroupingPolicy)>> {
            (self.list_grouping_fn.as_ref().unwrap())(subject)
        }
    }

    fn test_policy() -> Policy {
        Policy {
            scope: Scope::System,
            resource: Resource::User(Target::All),
            act: Act::Read,
        }
    }

    fn test_grouping() -> GroupingPolicy {
        GroupingPolicy {
            role: RoleName::new("admin").unwrap(),
            scope: Scope::System,
        }
    }

    fn make_uc(svc: StubPermissionService) -> PermissionUseCases<StubPermissionService> {
        PermissionUseCases::new(Arc::new(svc))
    }

    #[tokio::test]
    async fn add_policy_success() {
        let mut svc = StubPermissionService::default();
        svc.add_policy_fn = Some(Box::new(|_, _| Ok(true)));

        let uc = make_uc(svc);
        let uid = UserId::generate();
        assert!(uc.add_policy(uid, test_policy()).await.unwrap());
    }

    #[tokio::test]
    async fn remove_policy_success() {
        let mut svc = StubPermissionService::default();
        svc.remove_policy_fn = Some(Box::new(|_, _| Ok(true)));

        let uc = make_uc(svc);
        let uid = UserId::generate();
        assert!(uc.remove_policy(uid, test_policy()).await.unwrap());
    }

    #[tokio::test]
    async fn list_policies() {
        let mut svc = StubPermissionService::default();
        svc.list_policies_fn = Some(Box::new(|_| Ok(vec![])));

        let uc = make_uc(svc);
        let result = uc.list_policies(None).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn add_grouping_policy_success() {
        let mut svc = StubPermissionService::default();
        svc.add_grouping_fn = Some(Box::new(|_, _| Ok(true)));

        let uc = make_uc(svc);
        let uid = UserId::generate();
        assert!(uc.add_grouping_policy(uid, test_grouping()).await.unwrap());
    }

    #[tokio::test]
    async fn remove_grouping_policy_success() {
        let mut svc = StubPermissionService::default();
        svc.remove_grouping_fn = Some(Box::new(|_, _| Ok(true)));

        let uc = make_uc(svc);
        let uid = UserId::generate();
        assert!(uc.remove_grouping_policy(uid, test_grouping()).await.unwrap());
    }

    #[tokio::test]
    async fn list_grouping_policies() {
        let mut svc = StubPermissionService::default();
        svc.list_grouping_fn = Some(Box::new(|_| Ok(vec![])));

        let uc = make_uc(svc);
        let result = uc.list_grouping_policies(None).await.unwrap();
        assert!(result.is_empty());
    }
}
