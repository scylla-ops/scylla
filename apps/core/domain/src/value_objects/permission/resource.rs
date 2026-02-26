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

impl std::str::FromStr for Resource {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            None if s == "*" => Ok(Resource::All),
            Some(("user", "*")) => Ok(Resource::User(Target::All)),
            Some(("user", id)) => Ok(Resource::User(Target::Single(UserId::new(id.to_string())))),
            Some(("project", "*")) => Ok(Resource::Project(Target::All)),
            Some(("project", id)) => Ok(Resource::Project(Target::Single(ProjectId::new(
                id.to_string(),
            )))),
            Some(("organization", "*")) => Ok(Resource::Organization(Target::All)),
            Some(("organization", id)) => Ok(Resource::Organization(Target::Single(
                OrganizationId::new(id.to_string()),
            ))),
            Some(("pipeline", "*")) => Ok(Resource::Pipeline(Target::All)),
            Some(("pipeline", id)) => Ok(Resource::Pipeline(Target::Single(PipelineId::new(
                id.to_string(),
            )))),
            _ => Err(()),
        }
    }
}
