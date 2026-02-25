use crate::entities::{OrganizationId, ProjectId};

#[derive(Debug)]
pub enum Scope {
    System,
    Org(OrganizationId),
    Project {
        org: OrganizationId,
        project: ProjectId,
    },
}

impl Scope {
    pub fn as_str(&self) -> String {
        match self {
            Scope::System => "system".to_string(),
            Scope::Org(org) => format!("org/{}", org),
            Scope::Project { org, project } => format!("org/{}/{}", org, project),
        }
    }
}
