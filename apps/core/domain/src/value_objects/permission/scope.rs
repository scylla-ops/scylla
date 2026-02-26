use crate::entities::{OrganizationId, ProjectId, UserId};

#[derive(Debug)]
pub enum Scope {
    System,
    Org(OrganizationId),
    Project(ProjectId),
    User(UserId),
    All,
}

impl Scope {
    pub fn as_str(&self) -> String {
        match self {
            Scope::System => "system".to_string(),
            Scope::Org(org) => format!("org/{}", org),
            Scope::Project(project) => format!("project/{}", project),
            Scope::User(user) => format!("user/{}", user),
            Scope::All => "*".to_string(),
        }
    }
}
