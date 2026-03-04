use crate::value_objects::permission::{Act, Resource, Scope, Target};
use crate::value_objects::role::name::RoleName;
use derive_more::Constructor;

#[derive(Constructor, Debug)]
pub struct GroupingPolicy {
    pub role: RoleName,
    pub scope: Scope,
}
#[derive(Constructor, Debug)]
pub struct Policy {
    pub scope: Scope,
    pub resource: Resource,
    pub act: Act,
}

impl Policy {
    pub fn absolute() -> Self {
        Policy::new(Scope::All, Resource::All, Act::All)
    }
}

pub mod user {
    use super::*;
    use crate::entities::UserId;

    pub fn create() -> Policy {
        Policy::new(Scope::System, Resource::User(Target::All), Act::Create)
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

pub mod project {
    use super::*;
    use crate::entities::{OrganizationId, ProjectId, UserId};

    pub fn create(organization_id: OrganizationId) -> Policy {
        Policy::new(
            Scope::Org(organization_id),
            Resource::Project(Target::All),
            Act::Create,
        )
    }

    pub fn delete(project_id: ProjectId) -> Policy {
        Policy::new(
            Scope::Org(OrganizationId::new("*".to_string())),
            Resource::Project(Target::Single(project_id)),
            Act::Delete,
        )
    }

    pub fn update(project_id: ProjectId) -> Policy {
        Policy::new(
            Scope::Org(OrganizationId::new("*".to_string())),
            Resource::Project(Target::Single(project_id)),
            Act::Write,
        )
    }

    pub fn toggle_active(project_id: ProjectId) -> Policy {
        update(project_id)
    }

    pub fn get(project_id: ProjectId) -> Policy {
        Policy::new(
            Scope::Org(OrganizationId::new("*".to_string())),
            Resource::Project(Target::Single(project_id)),
            Act::Read,
        )
    }

    pub fn list() -> Policy {
        Policy::new(
            Scope::Org(OrganizationId::new("*".to_string())),
            Resource::Project(Target::All),
            Act::Read,
        )
    }

    pub fn list_users(project_id: ProjectId) -> Policy {
        Policy::new(
            Scope::Project(project_id),
            Resource::User(Target::All),
            Act::Read,
        )
    }

    pub fn list_user_projects(user_id: UserId) -> Policy {
        Policy::new(
            Scope::User(user_id),
            Resource::Project(Target::All),
            Act::Read,
        )
    }

    pub fn add_user_to_project(project_id: ProjectId) -> Policy {
        Policy::new(
            Scope::Project(project_id),
            Resource::User(Target::All),
            Act::Write,
        )
    }

    pub fn remove_user_from_project(project_id: ProjectId) -> Policy {
        Policy::new(
            Scope::Project(project_id),
            Resource::User(Target::All),
            Act::Delete,
        )
    }
}

pub mod pipeline {
    use super::*;
    use crate::entities::{OrganizationId, PipelineId, ProjectId};

    pub fn create(project_id: ProjectId) -> Policy {
        Policy::new(
            Scope::Project(project_id),
            Resource::Pipeline(Target::All),
            Act::Create,
        )
    }

    pub fn get(pipeline_id: PipelineId) -> Policy {
        Policy::new(
            Scope::Project(ProjectId::new("*".to_string())),
            Resource::Pipeline(Target::Single(pipeline_id)),
            Act::Read,
        )
    }

    pub fn update(pipeline_id: PipelineId) -> Policy {
        Policy::new(
            Scope::Project(ProjectId::new("*".to_string())),
            Resource::Pipeline(Target::Single(pipeline_id)),
            Act::Write,
        )
    }

    pub fn delete(pipeline_id: PipelineId) -> Policy {
        Policy::new(
            Scope::Project(ProjectId::new("*".to_string())),
            Resource::Pipeline(Target::Single(pipeline_id)),
            Act::Delete,
        )
    }

    pub fn list() -> Policy {
        Policy::new(
            Scope::Project(ProjectId::new("*".to_string())),
            Resource::Pipeline(Target::All),
            Act::Read,
        )
    }

    pub fn list_by_project(project_id: ProjectId) -> Policy {
        Policy::new(
            Scope::Project(project_id),
            Resource::Pipeline(Target::All),
            Act::Read,
        )
    }

    pub fn list_by_organization(organization_id: OrganizationId) -> Policy {
        Policy::new(
            Scope::Org(organization_id),
            Resource::Pipeline(Target::All),
            Act::Read,
        )
    }
}

pub mod job {
    use super::*;
    use crate::entities::{JobId, OrganizationId, ProjectId};

    pub fn get(job_id: JobId) -> Policy {
        Policy::new(Scope::All, Resource::Job(Target::Single(job_id)), Act::Read)
    }

    pub fn delete(job_id: JobId) -> Policy {
        Policy::new(
            Scope::All,
            Resource::Job(Target::Single(job_id)),
            Act::Delete,
        )
    }

    pub fn list() -> Policy {
        Policy::new(Scope::All, Resource::Job(Target::All), Act::Read)
    }

    pub fn list_by_pipeline() -> Policy {
        Policy::new(Scope::All, Resource::Job(Target::All), Act::Read)
    }

    pub fn list_by_project(project_id: ProjectId) -> Policy {
        Policy::new(
            Scope::Project(project_id),
            Resource::Job(Target::All),
            Act::Read,
        )
    }

    pub fn list_by_organization(organization_id: OrganizationId) -> Policy {
        Policy::new(
            Scope::Org(organization_id),
            Resource::Job(Target::All),
            Act::Read,
        )
    }
}

pub mod organization {
    use super::*;
    use crate::entities::{OrganizationId, UserId};

    pub fn create() -> Policy {
        Policy::new(
            Scope::System,
            Resource::Organization(Target::All),
            Act::Create,
        )
    }

    pub fn get(organization_id: OrganizationId) -> Policy {
        Policy::new(
            Scope::System,
            Resource::Organization(Target::Single(organization_id)),
            Act::Read,
        )
    }

    pub fn update(organization_id: OrganizationId) -> Policy {
        Policy::new(
            Scope::System,
            Resource::Organization(Target::Single(organization_id)),
            Act::Write,
        )
    }

    pub fn toggle_active(organization_id: OrganizationId) -> Policy {
        update(organization_id)
    }

    pub fn delete(organization_id: OrganizationId) -> Policy {
        Policy::new(
            Scope::System,
            Resource::Organization(Target::Single(organization_id)),
            Act::Delete,
        )
    }

    pub fn list() -> Policy {
        Policy::new(
            Scope::System,
            Resource::Organization(Target::All),
            Act::Read,
        )
    }

    pub fn list_users(organization_id: OrganizationId) -> Policy {
        Policy::new(
            Scope::Org(organization_id),
            Resource::User(Target::All),
            Act::Read,
        )
    }

    pub fn list_user_orgs(user_id: UserId) -> Policy {
        Policy::new(
            Scope::User(user_id),
            Resource::Organization(Target::All),
            Act::Read,
        )
    }

    pub fn add_user_to_organization(organization_id: OrganizationId) -> Policy {
        Policy::new(
            Scope::Org(organization_id),
            Resource::User(Target::All),
            Act::Write,
        )
    }

    pub fn remove_user_from_organization(organization_id: OrganizationId) -> Policy {
        Policy::new(
            Scope::Org(organization_id),
            Resource::User(Target::All),
            Act::Delete,
        )
    }
}

pub mod permission {
    use super::*;

    /// Any write operation on permissions requires absolute (super-admin) access.
    pub fn manage() -> Policy {
        Policy::absolute()
    }

    /// Listing permission rules requires system-level read-all.
    pub fn list() -> Policy {
        Policy::new(Scope::System, Resource::All, Act::Read)
    }
}
