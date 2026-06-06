use crate::domain::entities::{AppId, JobId, OrganizationId, PipelineId, ProjectId, UserId};

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
    App(AppId),
}

impl ResourceRef {
    /// The lowercase resource-type tag (`"system"`, `"user"`, …) — the kind
    /// without the id. The single source for a permission's resource type (see
    /// [`crate::domain::value_objects::permission::Permission::resource_type`])
    /// and the audit `resource_kind`, so the tag can never drift from the variant.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User(_) => "user",
            Self::Organization(_) => "organization",
            Self::Project(_) => "project",
            Self::Pipeline(_) => "pipeline",
            Self::Job(_) => "job",
            Self::App(_) => "app",
        }
    }
}

/// Compact, human-readable label for audit logs (e.g. `pipeline:01h…`, `system`).
impl std::fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "{}", self.kind()),
            Self::User(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::Organization(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::Project(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::Pipeline(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::Job(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::App(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
        }
    }
}
