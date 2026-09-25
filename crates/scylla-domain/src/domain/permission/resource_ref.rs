use crate::domain::ids::{AppId, JobId, OrganizationId, PipelineId, ProjectId, SecretId, UserId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRef {
    System,
    User(UserId),
    Organization(OrganizationId),
    Project(ProjectId),
    Pipeline(PipelineId),
    Job(JobId),
    Secret(SecretId),
    App(AppId),
}

impl ResourceRef {
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User(_) => "user",
            Self::Organization(_) => "organization",
            Self::Project(_) => "project",
            Self::Pipeline(_) => "pipeline",
            Self::Job(_) => "job",
            Self::Secret(_) => "secret",
            Self::App(_) => "app",
        }
    }
}

impl std::fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "{}", self.kind()),
            Self::User(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::Organization(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::Project(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::Pipeline(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::Job(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::Secret(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
            Self::App(id) => write!(f, "{}:{}", self.kind(), id.as_str()),
        }
    }
}
