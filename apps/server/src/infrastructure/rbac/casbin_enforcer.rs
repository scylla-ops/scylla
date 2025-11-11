use crate::application::ports::RbacEnforcer;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::UserId;
use async_trait::async_trait;
use casbin::{CoreApi, Enforcer, RbacApi};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Casbin-based RBAC enforcer
pub struct CasbinRbacEnforcer {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl CasbinRbacEnforcer {
    pub fn new(enforcer: Enforcer) -> Self {
        Self {
            enforcer: Arc::new(RwLock::new(enforcer)),
        }
    }
}

#[async_trait]
impl RbacEnforcer for CasbinRbacEnforcer {
    async fn enforce(
        &self,
        user_id: &UserId,
        domain: &str,
        resource: &str,
        action: &str,
    ) -> DomainResult<bool> {
        let enforcer = self.enforcer.read().await;

        let allowed = enforcer
            .enforce((user_id.as_str(), domain, resource, action))
            .map_err(|e| DomainError::internal(format!("RBAC enforcement error: {}", e)))?;

        Ok(allowed)
    }

    async fn add_role_for_user(
        &self,
        user_id: &UserId,
        role: &str,
        domain: &str,
    ) -> DomainResult<()> {
        let mut enforcer = self.enforcer.write().await;

        enforcer
            .add_role_for_user(user_id.as_str(), role, Some(domain))
            .await
            .map_err(|e| DomainError::internal(format!("Failed to add role: {}", e)))?;

        Ok(())
    }

    async fn remove_role_for_user(
        &self,
        user_id: &UserId,
        role: &str,
        domain: &str,
    ) -> DomainResult<()> {
        let mut enforcer = self.enforcer.write().await;

        enforcer
            .delete_role_for_user(user_id.as_str(), role, Some(domain))
            .await
            .map_err(|e| DomainError::internal(format!("Failed to remove role: {}", e)))?;

        Ok(())
    }

    async fn get_roles_for_user(
        &self,
        user_id: &UserId,
        domain: &str,
    ) -> DomainResult<Vec<String>> {
        let enforcer = self.enforcer.read().await;

        let roles = enforcer
            .get_roles_for_user(user_id.as_str(), Some(domain))
            .into_iter()
            .collect();

        Ok(roles)
    }

    async fn get_users_for_role(&self, role: &str, domain: &str) -> DomainResult<Vec<UserId>> {
        let enforcer = self.enforcer.read().await;

        let users = enforcer
            .get_users_for_role(role, Some(domain))
            .into_iter()
            .map(UserId::new)
            .collect();

        Ok(users)
    }

    async fn has_role(&self, user_id: &UserId, role: &str, domain: &str) -> DomainResult<bool> {
        let enforcer = self.enforcer.read().await;

        Ok(enforcer.has_role_for_user(user_id.as_str(), role, Some(domain)))
    }
}
