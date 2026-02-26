use casbin::{CoreApi, DefaultModel, Enforcer};
use casbin::{MgmtApi, Result as CasbinResult};
use domain::entities::EntityId;
use domain::errors::DomainResult;
use domain::ports::services::permission_service::PermissionService;
use domain::value_objects::permission::policy::{GroupingPolicy, Policy};
use log::debug;
use surreal_casbin_adapter::SurrealAdapter;

pub struct CasbinPermissionService {
    enforcer: Enforcer,
}

const MODEL: &'static str = include_str!("../../../config/casbin/rbac_model.conf");

impl CasbinPermissionService {
    pub async fn new(adapter: SurrealAdapter) -> CasbinResult<Self> {
        let model = DefaultModel::from_str(MODEL).await?;
        let enforcer = Enforcer::new(model, adapter).await?;
        Ok(Self { enforcer })
    }
}
#[async_trait::async_trait]
impl PermissionService for CasbinPermissionService {
    async fn check(&self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
        let result = self
            .enforcer
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

    async fn add_policy(&mut self, sub: impl EntityId, policy: Policy) -> DomainResult<bool> {
        self.enforcer
            .add_policy(vec![
                sub.to_string(),
                policy.scope.as_str(),
                policy.resource.as_str(),
                policy.act.as_str().to_string(),
            ])
            .await
            .map_err(|_| domain::errors::DomainError::Internal("Failed to add policy".to_string()))
    }

    async fn add_grouping_policy(
        &mut self,
        sub: impl EntityId,
        policy: GroupingPolicy,
    ) -> DomainResult<bool> {
        self.enforcer
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
}
