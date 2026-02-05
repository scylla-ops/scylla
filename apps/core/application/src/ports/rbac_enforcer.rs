use domain::entities::{OrganizationId, ProjectId, UserId};
use domain::errors::DomainResult;

/// Port for RBAC (Role-Based Access Control) enforcement
pub trait RbacEnforcer: Send + Sync {
    /// Check if a user has permission to perform an action on a resource
    fn enforce(
        &self,
        user_id: &UserId,
        domain: &str,
        resource: &str,
        action: &str,
    ) -> impl Future<Output = DomainResult<bool>> + Send;

    /// Add a role for a user in a specific domain
    fn add_role_for_user(
        &self,
        user_id: &UserId,
        role: &str,
        domain: &str,
    ) -> impl Future<Output = DomainResult<()>> + Send;

    /// Remove a role from a user in a specific domain
    fn remove_role_for_user(
        &self,
        user_id: &UserId,
        role: &str,
        domain: &str,
    ) -> impl Future<Output = DomainResult<()>> + Send;

    /// Get all roles for a user in a specific domain
    fn get_roles_for_user(
        &self,
        user_id: &UserId,
        domain: &str,
    ) -> impl Future<Output = DomainResult<Vec<String>>> + Send;

    /// Get all users with a specific role in a domain
    fn get_users_for_role(
        &self,
        role: &str,
        domain: &str,
    ) -> impl Future<Output = DomainResult<Vec<UserId>>> + Send;

    /// Check if user has a role in a domain
    fn has_role(
        &self,
        user_id: &UserId,
        role: &str,
        domain: &str,
    ) -> impl Future<Output = DomainResult<bool>> + Send;
}

/// Extension trait for organization-scoped RBAC operations
pub trait OrganizationRbacExt: RbacEnforcer {
    fn enforce_organization(
        &self,
        user_id: &UserId,
        org_id: &OrganizationId,
        resource: &str,
        action: &str,
    ) -> impl Future<Output = DomainResult<bool>> + Send {
        self.enforce(user_id, org_id.as_str(), resource, action)
    }

    fn add_organization_role(
        &self,
        user_id: &UserId,
        role: &str,
        org_id: &OrganizationId,
    ) -> impl Future<Output = DomainResult<()>> + Send {
        self.add_role_for_user(user_id, role, org_id.as_str())
    }
}

/// Extension trait for project-scoped RBAC operations
pub trait ProjectRbacExt: RbacEnforcer {
    fn enforce_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        resource: &str,
        action: &str,
    ) -> impl Future<Output = DomainResult<bool>> + Send {
        self.enforce(user_id, project_id.as_str(), resource, action)
    }

    fn add_project_role(
        &self,
        user_id: &UserId,
        role: &str,
        project_id: &ProjectId,
    ) -> impl Future<Output = DomainResult<()>> + Send {
        self.add_role_for_user(user_id, role, project_id.as_str())
    }
}

// Blanket implementations for all RbacEnforcer implementors
impl<T: RbacEnforcer + ?Sized> OrganizationRbacExt for T {}
impl<T: RbacEnforcer + ?Sized> ProjectRbacExt for T {}
