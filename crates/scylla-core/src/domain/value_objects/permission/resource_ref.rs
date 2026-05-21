use crate::domain::entities::{
    AgentId, AppId, JobId, OrganizationId, PipelineId, ProjectId, UserId,
};

/// A concrete resource an action targets, expressed in domain terms.
///
/// The Cedar adapter maps each variant to a typed entity UID (and loads its
/// ancestor chain). `System` is the singleton root used by cross-tenant /
/// global operations (list-all, manage grants) — admin-only in practice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRef {
    System,
    User(UserId),
    Organization(OrganizationId),
    Project(ProjectId),
    Pipeline(PipelineId),
    Job(JobId),
    Agent(AgentId),
    App(AppId),
}

/// Compact, human-readable label for audit logs (e.g. `pipeline:01h…`, `system`).
impl std::fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::User(id) => write!(f, "user:{}", id.as_str()),
            Self::Organization(id) => write!(f, "organization:{}", id.as_str()),
            Self::Project(id) => write!(f, "project:{}", id.as_str()),
            Self::Pipeline(id) => write!(f, "pipeline:{}", id.as_str()),
            Self::Job(id) => write!(f, "job:{}", id.as_str()),
            Self::Agent(id) => write!(f, "agent:{}", id.as_str()),
            Self::App(id) => write!(f, "app:{}", id.as_str()),
        }
    }
}
