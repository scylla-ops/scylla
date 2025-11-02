use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{OrganizationId, ProjectId, UserId};
use async_trait::async_trait;

/// Port for RBAC (Role-Based Access Control) enforcement
#[async_trait]
pub trait RbacEnforcer: Send + Sync {
    /// Check if a user has permission to perform an action on a resource
    async fn enforce(
        &self,
        user_id: &UserId,
        domain: &str,
        resource: &str,
        action: &str,
    ) -> DomainResult<bool>;

    /// Add a role for a user in a specific domain
    async fn add_role_for_user(
        &self,
        user_id: &UserId,
        role: &str,
        domain: &str,
    ) -> DomainResult<()>;

    /// Remove a role from a user in a specific domain
    async fn remove_role_for_user(
        &self,
        user_id: &UserId,
        role: &str,
        domain: &str,
    ) -> DomainResult<()>;

    /// Get all roles for a user in a specific domain
    async fn get_roles_for_user(&self, user_id: &UserId, domain: &str)
    -> DomainResult<Vec<String>>;

    /// Get all users with a specific role in a domain
    async fn get_users_for_role(&self, role: &str, domain: &str) -> DomainResult<Vec<UserId>>;

    /// Check if user has a role in a domain
    async fn has_role(&self, user_id: &UserId, role: &str, domain: &str) -> DomainResult<bool>;
}

/// Convenience methods for organization-scoped permissions
#[async_trait]
pub trait OrganizationRbacExt: RbacEnforcer {
    async fn enforce_organization(
        &self,
        user_id: &UserId,
        org_id: &OrganizationId,
        resource: &str,
        action: &str,
    ) -> DomainResult<bool> {
        self.enforce(user_id, org_id.as_str(), resource, action)
            .await
    }

    async fn add_organization_role(
        &self,
        user_id: &UserId,
        role: &str,
        org_id: &OrganizationId,
    ) -> DomainResult<()> {
        self.add_role_for_user(user_id, role, org_id.as_str()).await
    }
}

/// Convenience methods for project-scoped permissions
#[async_trait]
pub trait ProjectRbacExt: RbacEnforcer {
    async fn enforce_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        resource: &str,
        action: &str,
    ) -> DomainResult<bool> {
        self.enforce(user_id, project_id.as_str(), resource, action)
            .await
    }

    async fn add_project_role(
        &self,
        user_id: &UserId,
        role: &str,
        project_id: &ProjectId,
    ) -> DomainResult<()> {
        self.add_role_for_user(user_id, role, project_id.as_str())
            .await
    }
}

// Blanket implementations for all RbacEnforcer implementors
impl<T: RbacEnforcer + ?Sized> OrganizationRbacExt for T {}
impl<T: RbacEnforcer + ?Sized> ProjectRbacExt for T {}
