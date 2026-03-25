use crate::domain::entities::{OrganizationId, ProjectId, UserId};

#[derive(Debug)]
pub enum Scope {
    System,
    Org(OrganizationId),
    Project(ProjectId),
    User(UserId),
    All,
}

impl Scope {
    #[must_use] 
    pub fn as_str(&self) -> String {
        match self {
            Scope::System => "system".to_string(),
            Scope::Org(org) => format!("org/{org}"),
            Scope::Project(project) => format!("project/{project}"),
            Scope::User(user) => format!("user/{user}"),
            Scope::All => "*".to_string(),
        }
    }
}

impl std::str::FromStr for Scope {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            None if s == "system" => Ok(Scope::System),
            None if s == "*" => Ok(Scope::All),
            Some(("org", id)) => Ok(Scope::Org(OrganizationId::new(id.to_string()))),
            Some(("project", id)) => Ok(Scope::Project(ProjectId::new(id.to_string()))),
            Some(("user", id)) => Ok(Scope::User(UserId::new(id.to_string()))),
            _ => Err(()),
        }
    }
}
