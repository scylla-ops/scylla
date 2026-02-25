use crate::entities::{EntityId, PipelineId, UserId};

#[derive(Debug)]
pub enum Resource {
    User(Target<UserId>),
    Org,
    Project,
    Pipeline(Target<PipelineId>),
}

#[derive(Debug)]
pub enum Target<T: EntityId> {
    None,
    All,
    Single(T),
}

impl Resource {
    pub fn as_str(&self) -> String {
        match self {
            Resource::User(Target::All) => "user/*".to_string(),
            Resource::User(Target::None) => "user".to_string(),
            Resource::User(Target::Single(_user_id)) => format!("user/{}", "user_id"),
            Resource::Org => "org".to_string(),
            Resource::Project => "project".to_string(),
            Resource::Pipeline(Target::All) => "pipeline/*".to_string(),
            Resource::Pipeline(Target::Single(id)) => format!("pipeline/{}", id),
            Resource::Pipeline(Target::None) => "pipeline".to_string(),
        }
    }
}
