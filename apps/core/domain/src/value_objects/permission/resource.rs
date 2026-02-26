use crate::entities::{EntityId, OrganizationId, PipelineId, ProjectId, UserId};

#[derive(Debug)]
pub enum Resource {
    User(Target<UserId>),
    Pipeline(Target<PipelineId>),
    Project(Target<ProjectId>),
    Organization(Target<OrganizationId>),
    All,
}

#[derive(Debug)]
pub enum Target<T: EntityId> {
    All,
    Single(T),
}

impl Resource {
    pub fn as_str(&self) -> String {
        match self {
            Resource::User(Target::All) => "user/*".to_string(),
            Resource::User(Target::Single(id)) => format!("user/{}", id),
            Resource::Pipeline(Target::All) => "pipeline/*".to_string(),
            Resource::Pipeline(Target::Single(id)) => format!("pipeline/{}", id),
            Resource::All => "*".to_string(),
            Resource::Project(Target::All) => "project/*".to_string(),
            Resource::Project(Target::Single(id)) => format!("project/{}", id),
            Resource::Organization(Target::All) => "organization/*".to_string(),
            Resource::Organization(Target::Single(id)) => format!("organization/{}", id),
        }
    }
}
