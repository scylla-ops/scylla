use crate::value_objects::permission::{Act, Resource, Scope, Target};
use derive_more::Constructor;

#[derive(Constructor, Debug)]
pub struct Policy {
    pub scope: Scope,
    pub resource: Resource,
    pub act: Act,
}

pub mod user {
    use super::*;
    use crate::entities::UserId;

    pub fn create() -> Policy {
        Policy::new(Scope::System, Resource::User(Target::None), Act::Create)
    }
    pub fn get(user_id: UserId) -> Policy {
        Policy::new(
            Scope::System,
            Resource::User(Target::Single(user_id)),
            Act::Read,
        )
    }
    pub fn get_all() -> Policy {
        Policy::new(Scope::System, Resource::User(Target::All), Act::Read)
    }
    pub fn delete(user_id: UserId) -> Policy {
        Policy::new(
            Scope::System,
            Resource::User(Target::Single(user_id)),
            Act::Delete,
        )
    }
    pub fn update(user_id: UserId) -> Policy {
        Policy::new(
            Scope::System,
            Resource::User(Target::Single(user_id)),
            Act::Write,
        )
    }
}

pub mod pipeline {
    use super::*;
    use crate::entities::{OrganizationId, PipelineId, ProjectId};

    pub fn delete(org: OrganizationId, project: ProjectId, pipeline_id: PipelineId) -> Policy {
        Policy::new(
            Scope::Project { org, project },
            Resource::Pipeline(Target::Single(pipeline_id)),
            Act::Delete,
        )
    }

    pub fn create(org: OrganizationId, project: ProjectId) -> Policy {
        Policy::new(
            Scope::Project { org, project },
            Resource::Pipeline(Target::None),
            Act::Create,
        )
    }
}
