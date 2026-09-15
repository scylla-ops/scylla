use crate::authz::role::RoleRepository;
use crate::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::role::RoleName;

pub const SYSTEM_ADMIN_ROLE: &str = "system-admin";
pub const ORGANIZATION_ADMIN_ROLE: &str = "organization-admin";
pub const PROJECT_ADMIN_ROLE: &str = "project-admin";
pub const ORGANIZATION_AGENT_ROLE: &str = "organization-agent";
pub const PROJECT_AGENT_ROLE: &str = "project-agent";
pub const ORGANIZATION_TRIGGER_RUNNER_ROLE: &str = "organization-trigger-runner";
pub const ORGANIZATION_VIEWER_ROLE: &str = "organization-viewer";
pub const ORGANIZATION_MEMBER_ROLE: &str = "organization-member";
pub const PROJECT_DEVELOPER_ROLE: &str = "project-developer";
pub const PROJECT_VIEWER_ROLE: &str = "project-viewer";

/// A scope must keep one owner; includes `system-admin` so the last operator cannot lock everyone out.
#[must_use]
pub fn is_owner_role(role: &RoleName) -> bool {
    matches!(
        role.as_str(),
        SYSTEM_ADMIN_ROLE | ORGANIZATION_ADMIN_ROLE | PROJECT_ADMIN_ROLE
    )
}

#[must_use]
pub fn removal_orphans_scope(grants: &[Grant], scope: &Scope, victim: &Principal) -> bool {
    let victim_owns_here = grants
        .iter()
        .any(|g| &g.principal == victim && &g.scope == scope && is_owner_role(&g.role));
    if !victim_owns_here {
        return false;
    }
    !grants.iter().any(|g| {
        &g.scope == scope
            && &g.principal != victim
            && matches!(g.principal, Principal::User(_))
            && is_owner_role(&g.role)
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    System,
    Organization(OrganizationId),
    Project(ProjectId),
}

impl Scope {
    #[must_use]
    pub fn kind(&self) -> ScopeKind {
        match self {
            Self::System => ScopeKind::System,
            Self::Organization(_) => ScopeKind::Organization,
            Self::Project(_) => ScopeKind::Project,
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Organization(id) => write!(f, "organization:{}", id.as_str()),
            Self::Project(id) => write!(f, "project:{}", id.as_str()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    System,
    Organization,
    Project,
}

impl ScopeKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Organization => "organization",
            Self::Project => "project",
        }
    }

    fn depth(self) -> u8 {
        match self {
            Self::System => 0,
            Self::Organization => 1,
            Self::Project => 2,
        }
    }

    #[must_use]
    pub fn covers(self, inner: ScopeKind) -> bool {
        self.depth() <= inner.depth()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleKind {
    Admin,
    Member,
    Agent,
}

impl RoleKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Agent => "agent",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GrantableRole {
    pub name: &'static str,
    pub scope: ScopeKind,
    pub kind: RoleKind,
    pub description: &'static str,
}

pub const GRANTABLE_ROLES: &[GrantableRole] = &[
    GrantableRole {
        name: SYSTEM_ADMIN_ROLE,
        scope: ScopeKind::System,
        kind: RoleKind::Admin,
        description: "Global super-user: full control over every scope.",
    },
    GrantableRole {
        name: ORGANIZATION_ADMIN_ROLE,
        scope: ScopeKind::Organization,
        kind: RoleKind::Admin,
        description: "Owner of an organization and everything beneath it.",
    },
    GrantableRole {
        name: ORGANIZATION_AGENT_ROLE,
        scope: ScopeKind::Organization,
        kind: RoleKind::Agent,
        description: "Machine app scoped to an organization: pull and run its jobs.",
    },
    GrantableRole {
        name: PROJECT_ADMIN_ROLE,
        scope: ScopeKind::Project,
        kind: RoleKind::Admin,
        description: "Owner of a project and everything beneath it.",
    },
    GrantableRole {
        name: PROJECT_AGENT_ROLE,
        scope: ScopeKind::Project,
        kind: RoleKind::Agent,
        description: "Machine app scoped to a project: pull and run its jobs.",
    },
    GrantableRole {
        name: ORGANIZATION_TRIGGER_RUNNER_ROLE,
        scope: ScopeKind::Organization,
        kind: RoleKind::Agent,
        description: "Machine app that fires the triggers of an organization: run its pipelines.",
    },
    GrantableRole {
        name: ORGANIZATION_MEMBER_ROLE,
        scope: ScopeKind::Organization,
        kind: RoleKind::Member,
        description: "Belongs to the organization: sees it exists, nothing more.",
    },
    GrantableRole {
        name: ORGANIZATION_VIEWER_ROLE,
        scope: ScopeKind::Organization,
        kind: RoleKind::Member,
        description: "Read every project and run in the organization, change nothing.",
    },
    GrantableRole {
        name: PROJECT_DEVELOPER_ROLE,
        scope: ScopeKind::Project,
        kind: RoleKind::Member,
        description: "Build in a project: create, edit and run its pipelines.",
    },
    GrantableRole {
        name: PROJECT_VIEWER_ROLE,
        scope: ScopeKind::Project,
        kind: RoleKind::Member,
        description: "Read a project, its pipelines and its runs, change nothing.",
    },
];

#[must_use]
pub fn grantable_roles(filter: Option<ScopeKind>) -> Vec<GrantableRole> {
    GRANTABLE_ROLES
        .iter()
        .copied()
        .filter(|r| filter.is_none_or(|k| r.scope == k))
        .collect()
}

/// Shared by `CreateGrant` and the invitation flow so what can be granted equals what can be invited.
pub async fn validate_role_in_db(
    role_repo: &dyn RoleRepository,
    role: &RoleName,
    scope: &Scope,
) -> DomainResult<()> {
    let found = role_repo
        .get(role.as_str())
        .await?
        .ok_or_else(|| DomainError::validation(format!("unknown role '{}'", role.as_str())))?;
    if found.scope != scope.kind() {
        return Err(DomainError::validation(format!(
            "role '{}' is grantable only on {} scope, not {}",
            role.as_str(),
            found.scope.as_str(),
            scope.kind().as_str()
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    User(UserId),
    App(AppId),
}

impl Principal {
    #[must_use]
    pub fn from_caller(caller: &CallerContext) -> Option<Self> {
        match caller {
            CallerContext::User(id) => Some(Self::User(id.clone())),
            CallerContext::App(id) => Some(Self::App(id.clone())),
            CallerContext::Service(_) | CallerContext::Anonymous => None,
        }
    }

    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::User(_) => "user",
            Self::App(_) => "app",
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::User(id) => id.as_str(),
            Self::App(id) => id.as_str(),
        }
    }
}

impl std::fmt::Display for Principal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.kind(), self.id())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub id: String,
    pub principal: Principal,
    pub role: RoleName,
    pub scope: Scope,
}

impl Grant {
    #[must_use]
    pub fn new(principal: Principal, role: RoleName, scope: Scope) -> Self {
        Self {
            id: scylla_domain::domain::ids::new_id(),
            principal,
            role,
            scope,
        }
    }
}

mod repository;

pub use repository::*;
