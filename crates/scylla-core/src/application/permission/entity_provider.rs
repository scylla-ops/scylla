use crate::domain::entities::{OrganizationId, PipelineId, ProjectId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::ResourceRef;
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;

/// Everything Cedar needs to know about the *principal* at check time:
/// the roles it holds (→ entity parents `User in Role`) and the orgs/projects
/// it belongs to (→ ABAC attributes `principal.memberOrgs` / `memberProjects`).
#[derive(Debug, Default, Clone)]
pub struct PrincipalAuthz {
    pub roles: Vec<RoleName>,
    pub member_orgs: Vec<OrganizationId>,
    pub member_projects: Vec<ProjectId>,
}

/// The ancestor chain of a *resource* (→ entity parents, e.g.
/// `Pipeline in Project in Organization`). Only the levels that exist for a
/// given resource are populated; missing levels are `None`.
#[derive(Debug, Default, Clone)]
pub struct ResourceAncestors {
    pub organization: Option<OrganizationId>,
    pub project: Option<ProjectId>,
    pub pipeline: Option<PipelineId>,
}

/// Read-only port the Cedar adapter uses to materialise entities + relationships
/// for an authorization request. Implemented in infra over the existing
/// `user_roles` / `user_organization` / `user_project` tables and the
/// pipeline→project→org foreign keys.
#[async_trait]
pub trait AuthzEntityProvider: Send + Sync {
    /// Roles + memberships for a user principal.
    async fn principal_authz(&self, user: &UserId) -> DomainResult<PrincipalAuthz>;

    /// Ancestor chain for a resource. For `System` / `User` / `Agent`
    /// resources (no tenancy parents) this returns an empty `ResourceAncestors`.
    async fn resource_ancestors(&self, resource: &ResourceRef) -> DomainResult<ResourceAncestors>;
}
