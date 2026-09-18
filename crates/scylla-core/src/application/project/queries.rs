//! One struct per read. A query has a permission and an output, and no staged value.

use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::ids::{OrganizationId, ProjectId, UserId};
use crate::domain::permission::Permission;
use crate::domain::project::Project;
use crate::domain::user::User;
use scylla_extension::{Describe, Query, Read};

#[derive(Debug)]
pub struct GetProject {
    pub id: ProjectId,
}

impl Describe for GetProject {
    type Path = Read;

    fn permission(&self) -> Permission {
        Permission::ReadProject(self.id.clone())
    }
}

impl Query for GetProject {
    type Output = Project;
}

#[derive(Debug)]
pub struct ListProjects {
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListProjects {
    type Path = Read;

    fn permission(&self) -> Permission {
        Permission::ListProjects
    }
}

impl Query for ListProjects {
    type Output = PaginatedResult<Project>;
}

/// Gated on `readOrganization`, not on `listProjectsByOrganization`: a project-only role must
/// see its own project, not be refused. The wider permission only widens the visible set.
#[derive(Debug)]
pub struct ListOrganizationProjects {
    pub organization_id: OrganizationId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListOrganizationProjects {
    type Path = Read;

    fn permission(&self) -> Permission {
        Permission::ReadOrganization(self.organization_id.clone())
    }
}

impl Query for ListOrganizationProjects {
    type Output = PaginatedResult<Project>;
}

#[derive(Debug)]
pub struct ListProjectMembers {
    pub project_id: ProjectId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListProjectMembers {
    type Path = Read;

    fn permission(&self) -> Permission {
        Permission::ListProjectMembers(self.project_id.clone())
    }
}

impl Query for ListProjectMembers {
    type Output = PaginatedResult<User>;
}

#[derive(Debug)]
pub struct ListUserProjects {
    pub user_id: UserId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListUserProjects {
    type Path = Read;

    fn permission(&self) -> Permission {
        Permission::ListUserProjects(self.user_id.clone())
    }
}

impl Query for ListUserProjects {
    type Output = PaginatedResult<Project>;
}
