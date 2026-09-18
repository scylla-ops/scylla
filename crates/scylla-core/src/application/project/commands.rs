//! One struct per write. The permission is declared next to the fields it names, and the two
//! payload types say what exists: a `Draft` is not in the store, a `Project` is, a `Deleted` was.

use crate::domain::ids::{OrganizationId, ProjectId};
use crate::domain::permission::Permission;
use crate::domain::project::{Project, ProjectDescription, ProjectName};
use scylla_auth::authz::Grant;
use scylla_extension::{Command, Deleted, Draft};

pub struct CreateProject {
    pub organization_id: OrganizationId,
    pub name: ProjectName,
    pub description: Option<ProjectDescription>,
}

/// For a user caller, the grant that makes them the project's admin; both rows go in one
/// transaction, so they are staged together.
pub struct NewProject {
    pub project: Project,
    pub owner: Option<Grant>,
}

impl Command for CreateProject {
    type Staged = Draft<NewProject>;
    type Committed = Project;

    fn permission(&self) -> Permission {
        Permission::CreateProject(self.organization_id.clone())
    }
}

pub struct UpdateProject {
    pub id: ProjectId,
    pub name: Option<ProjectName>,
    pub description: Option<Option<ProjectDescription>>,
}

impl Command for UpdateProject {
    type Staged = Draft<Project>;
    type Committed = Project;

    fn permission(&self) -> Permission {
        Permission::UpdateProject(self.id.clone())
    }
}

pub struct SetProjectActive {
    pub id: ProjectId,
    pub is_active: bool,
}

impl Command for SetProjectActive {
    type Staged = Draft<Project>;
    type Committed = Project;

    fn permission(&self) -> Permission {
        Permission::UpdateProject(self.id.clone())
    }
}

pub struct DeleteProject {
    pub id: ProjectId,
}

impl Command for DeleteProject {
    type Staged = Project;
    type Committed = Deleted<Project>;

    fn permission(&self) -> Permission {
        Permission::DeleteProject(self.id.clone())
    }
}
