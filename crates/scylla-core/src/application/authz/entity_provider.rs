use crate::domain::entities::{AppId, OrganizationId, PipelineId, ProjectId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::ResourceRef;
use async_trait::async_trait;

/// Everything Cedar needs to know about the *principal* at check time: the
/// orgs/projects it belongs to (→ ABAC attributes `principal.memberOrgs` /
/// `memberProjects`). Global authority is no longer a role here — it is a
/// System-scoped grant, linked as a template instance like any other grant.
#[derive(Debug, Default, Clone)]
pub struct PrincipalAuthz {
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
/// `user_organization` / `user_project` tables and the pipeline→project→org
/// foreign keys.
#[async_trait]
pub trait AuthzEntityProvider: Send + Sync {
    /// Roles + memberships for a user principal.
    async fn principal_authz(&self, user: &UserId) -> DomainResult<PrincipalAuthz>;

    /// Ancestor chain for a resource. For `System` / `User` resources (no
    /// tenancy parents) this returns an empty `ResourceAncestors`.
    async fn resource_ancestors(&self, resource: &ResourceRef) -> DomainResult<ResourceAncestors>;

    /// Whether a machine **App** principal is currently active: its row still
    /// exists *and* its `is_active` flag is set. Re-checked on every
    /// authorization so a disabled or deleted App is denied immediately — even
    /// over a long-lived stream opened while it was still active. An unknown id
    /// (deleted App) returns `false`.
    async fn app_is_active(&self, app: &AppId) -> DomainResult<bool>;
}
