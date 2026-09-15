use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, OrganizationId, PipelineId, ProjectId};
use crate::domain::permission::ResourceRef;
use async_trait::async_trait;

#[derive(Debug, Default, Clone)]
pub struct ResourceAncestors {
    pub organization: Option<OrganizationId>,
    pub project: Option<ProjectId>,
    pub pipeline: Option<PipelineId>,
}

#[async_trait]
pub trait AuthzEntityProvider: Send + Sync {
    async fn resource_ancestors(&self, resource: &ResourceRef) -> DomainResult<ResourceAncestors>;

    /// Checked on every authorization so a disabled or deleted App stops at once, even mid-stream.
    async fn app_is_active(&self, app: &AppId) -> DomainResult<bool>;
}
