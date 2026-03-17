use casbin::Result as CasbinResult;
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use domain::entities::EntityId;
use domain::errors::DomainResult;
use domain::ports::services::permission_service::PermissionService;
use domain::value_objects::permission::policy::{GroupingPolicy, Policy};
use domain::value_objects::permission::{Act, Resource, Scope};
use domain::value_objects::role::name::RoleName;
use log::debug;
use surreal_casbin_adapter::SurrealAdapter;
use tokio::sync::RwLock;

pub struct CasbinPermissionService {
    enforcer: RwLock<Enforcer>,
}

const MODEL: &str = include_str!("../../../config/casbin/rbac_model.conf");

impl CasbinPermissionService {
    pub async fn new(adapter: SurrealAdapter) -> CasbinResult<Self> {
        let model = DefaultModel::from_str(MODEL).await?;
        let enforcer = Enforcer::new(model, adapter).await?;
        Ok(Self {
            enforcer: RwLock::new(enforcer),
        })
    }
}

#[async_trait::async_trait]
impl PermissionService for CasbinPermissionService {
    async fn check(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
        let enforcer = self.enforcer.read().await;
        let result = enforcer
            .enforce((
                sub.as_ref(),
                policy.scope.as_str(),
                policy.resource.as_str(),
                policy.act.as_str(),
            ))
            .map_err(|e| domain::errors::DomainError::Internal(e.to_string()))?;

        if result {
            Ok(true)
        } else {
            debug!("Permission denied for subject {}: {:?}", sub, policy);
            Err(domain::errors::DomainError::Forbidden(
                "Permission denied".to_string(),
            ))
        }
    }

    async fn add_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .add_policy(vec![
                sub.to_string(),
                policy.scope.as_str(),
                policy.resource.as_str(),
                policy.act.as_str().to_string(),
            ])
            .await
            .map_err(|_| domain::errors::DomainError::Internal("Failed to add policy".to_string()))
    }

    async fn remove_policy(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .remove_policy(vec![
                sub.to_string(),
                policy.scope.as_str(),
                policy.resource.as_str(),
                policy.act.as_str().to_string(),
            ])
            .await
            .map_err(|_| {
                domain::errors::DomainError::Internal("Failed to remove policy".to_string())
            })
    }

    async fn list_policies(&self, subject: Option<&str>) -> DomainResult<Vec<(String, Policy)>> {
        let enforcer = self.enforcer.read().await;
        let policies = enforcer.get_policy();
        let result = policies
            .into_iter()
            .filter(|row| {
                if let Some(sub) = subject {
                    row.first().map(|s| s.as_str()) == Some(sub)
                } else {
                    true
                }
            })
            .filter_map(|row| {
                let get = |i: usize| row.get(i).cloned().unwrap_or_default();
                let sub = get(0);
                let scope: Scope = get(1).parse().ok()?;
                let resource: Resource = get(2).parse().ok()?;
                let act: Act = get(3).parse().ok()?;
                Some((sub, Policy::new(scope, resource, act)))
            })
            .collect();
        Ok(result)
    }

    async fn add_grouping_policy(
        &self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .add_grouping_policy(vec![
                sub.to_string(),
                policy.role.as_ref().to_string(),
                policy.scope.as_str().to_string(),
            ])
            .await
            .map_err(|_| {
                domain::errors::DomainError::Internal("Failed to add grouping policy".to_string())
            })
    }

    async fn remove_grouping_policy(
        &self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .remove_grouping_policy(vec![
                sub.to_string(),
                policy.role.as_ref().to_string(),
                policy.scope.as_str().to_string(),
            ])
            .await
            .map_err(|_| {
                domain::errors::DomainError::Internal(
                    "Failed to remove grouping policy".to_string(),
                )
            })
    }

    async fn list_grouping_policies(
        &self,
        subject: Option<&str>,
    ) -> DomainResult<Vec<(String, GroupingPolicy)>> {
        let enforcer = self.enforcer.read().await;
        let policies = enforcer.get_grouping_policy();
        let result = policies
            .into_iter()
            .filter(|row| {
                if let Some(sub) = subject {
                    row.first().map(|s| s.as_str()) == Some(sub)
                } else {
                    true
                }
            })
            .filter_map(|row| {
                let get = |i: usize| row.get(i).cloned().unwrap_or_default();
                let sub = get(0);
                let role = RoleName::new(get(1)).ok()?;
                let scope: Scope = get(2).parse().ok()?;
                Some((sub, GroupingPolicy::new(role, scope)))
            })
            .collect();
        Ok(result)
    }
}
