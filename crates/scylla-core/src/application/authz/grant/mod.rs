use crate::application::authz::role::RoleRepository;
use crate::domain::entities::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::is_known_permission;
use crate::domain::value_objects::role::RoleName;

// Canonical builtin role keys — the stable ids grants reference and the default
// seed inserts. Convention: `<scope>-<role>`, kebab-case, scope ∈ {system,
// organization, project}. The live Cedar policy bodies are generated per role
// from the `roles` table (see `cedar_permission_service`), not hard-coded here.
// The implicit `system-member` / `organization-member` / `project-member` tiers
// are NOT named roles: membership is ABAC (`user_organization` / `user_project`).

/// Global super-user (full control, every scope), via a grant on the System scope.
pub const SYSTEM_ADMIN_ROLE: &str = "system-admin";
/// Owner of an organization and everything beneath it.
pub const ORGANIZATION_ADMIN_ROLE: &str = "organization-admin";
/// Owner of a project and everything beneath it.
pub const PROJECT_ADMIN_ROLE: &str = "project-admin";
/// Restricted role for machine Apps (agents) scoped to an organization: only the
/// permissions needed to pull and execute jobs within that scope (read pipeline,
/// execute job, write job status/log), held in the role's `role_permissions`.
pub const ORGANIZATION_AGENT_ROLE: &str = "organization-agent";
/// Same restricted agent capability, scoped to a single project.
pub const PROJECT_AGENT_ROLE: &str = "project-agent";

/// Owner-equivalent roles: holding one grants full control over a scope. A scope
/// must never lose its last owner, so revoking one of these is guarded. Includes
/// `system-admin` so the last global admin can't be revoked into a lockout.
#[must_use]
pub(crate) fn is_owner_role(role: &RoleName) -> bool {
    matches!(
        role.as_str(),
        SYSTEM_ADMIN_ROLE | ORGANIZATION_ADMIN_ROLE | PROJECT_ADMIN_ROLE
    )
}

/// Whether removing every grant `victim` holds at `scope` would leave the scope
/// with no human owner. This is the membership-removal counterpart of the
/// per-grant last-owner guard inline in [`GrantUseCases::revoke`]: a scope must
/// always retain at least one *human* owner, so removing its sole owner-holding
/// member is blocked rather than orphaning the org/project. Returns false when
/// `victim` holds no owner role at `scope` (removing a non-owner never orphans
/// it) or when another `User` still holds one there. App owners never count as
/// the retained human owner, matching `revoke`.
#[must_use]
pub fn removal_orphans_scope(grants: &[Grant], scope: &Scope, victim: &Principal) -> bool {
    let victim_owns_here = grants.iter().any(|g| {
        &g.principal == victim
            && &g.scope == scope
            && matches!(&g.target, GrantTarget::Role(role) if is_owner_role(role))
    });
    if !victim_owns_here {
        return false;
    }
    !grants.iter().any(|g| {
        &g.scope == scope
            && &g.principal != victim
            && matches!(g.principal, Principal::User(_))
            && matches!(&g.target, GrantTarget::Role(role) if is_owner_role(role))
    })
}

/// The scope a grant is bound to. Maps to the `?resource` slot of the linked
/// Cedar template. `System` is the tenancy root (org ∈ System ∈ …): a grant
/// there — e.g. `system-admin` — covers everything beneath. It is the unified
/// replacement for the former global-role mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    System,
    Organization(OrganizationId),
    Project(ProjectId),
}

impl Scope {
    /// The id-free discriminant of this scope — used by the assignable-roles
    /// catalog and to validate a grant's (role, scope) pairing.
    #[must_use]
    pub fn kind(&self) -> ScopeKind {
        match self {
            Self::System => ScopeKind::System,
            Self::Organization(_) => ScopeKind::Organization,
            Self::Project(_) => ScopeKind::Project,
        }
    }
}

/// Which kind of scope a role/grant binds to, without the concrete id. The
/// catalog declares one of these per role; a grant is valid only when its role's
/// declared `ScopeKind` matches its scope.
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

    /// Containment depth in the scope hierarchy (System is broadest = 0).
    fn depth(self) -> u8 {
        match self {
            Self::System => 0,
            Self::Organization => 1,
            Self::Project => 2,
        }
    }

    /// Whether a grant at this scope covers resources whose home scope is
    /// `inner` — true when this scope is `inner` or one of its ancestors
    /// (System ⊃ Organization ⊃ Project). A grant authorises within its scope's
    /// subtree, so a permission is usable in a role iff the role's scope covers
    /// the permission's home scope.
    #[must_use]
    pub fn covers(self, inner: ScopeKind) -> bool {
        self.depth() <= inner.depth()
    }
}

/// Full-control admin role vs restricted machine-agent role. Used by the
/// compile-time grantable-role catalog ([`GRANTABLE_ROLES`]) to describe a
/// builtin; the live Cedar policy bodies are now generated per role from the
/// `roles` table (full control → unconstrained action; otherwise the role's
/// explicit permission keys), not from this kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleKind {
    Admin,
    Agent,
}

impl RoleKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Agent => "agent",
        }
    }
}

/// A role assignable through a grant: its name, the scope kind it must bind to,
/// whether it is an admin or agent role, and a human description. The catalog
/// ([`GRANTABLE_ROLES`]) is the single source of truth for what `CreateGrant`
/// accepts — there are no runtime-defined roles, so listing it is exhaustive.
#[derive(Debug, Clone, Copy)]
pub struct GrantableRole {
    pub name: &'static str,
    pub scope: ScopeKind,
    pub kind: RoleKind,
    pub description: &'static str,
}

/// Every role that can be granted. One entry per `*_ROLE` constant above — keep
/// the two in sync (the test below asserts each entry's name is a known const).
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
];

/// The assignable-role catalog, optionally narrowed to one scope kind. Pure /
/// static — no permission check (the names are compile-time constants, not
/// sensitive data) and no DB hit.
#[must_use]
pub fn grantable_roles(filter: Option<ScopeKind>) -> Vec<GrantableRole> {
    GRANTABLE_ROLES
        .iter()
        .copied()
        .filter(|r| filter.is_none_or(|k| r.scope == k))
        .collect()
}

/// Validate a role grant against the DB role catalog: the role must exist
/// (builtin *or* custom) and declare the scope kind the grant binds to (so e.g.
/// an `organization-admin` is rejected on a Project). The single role-grant
/// validity check — shared by `CreateGrant` and the invitation flow — so "what
/// can be granted" equals "what can be invited" by construction, and a persisted
/// grant can never name a role the Cedar adapter cannot link. A role created
/// through `RoleService` (builtin or custom) becomes grantable through the same
/// path; the rest of the pipeline (anti-escalation expansion, Cedar emission)
/// already resolves any role by id.
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

/// Validate a direct permission grant's key against the permission catalog, so a
/// persisted grant can never name a permission the Cedar adapter cannot emit.
pub fn validate_permission_key(key: &str) -> DomainResult<()> {
    if is_known_permission(key) {
        Ok(())
    } else {
        Err(DomainError::validation(format!(
            "unknown permission '{key}'"
        )))
    }
}

/// The principal a grant is bound to — a human `User` or a machine `App`. Maps
/// to the `(principal_kind, principal_id)` columns of `grants` and to
/// the Cedar `?principal` slot (`Scylla::User` / `Scylla::App`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    User(UserId),
    App(AppId),
}

impl Principal {
    /// Persistence discriminant — the `principal_kind` column value.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::User(_) => "user",
            Self::App(_) => "app",
        }
    }

    /// The principal's raw id — the `principal_id` column value.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::User(id) => id.as_str(),
            Self::App(id) => id.as_str(),
        }
    }
}

/// What a grant confers: a whole role, or a single permission. A direct
/// permission grant is additive to the principal's role-derived permissions
/// (e.g. "Alice may runPipeline within Org A" on top of whatever roles she has).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantTarget {
    /// A role, referenced by its id (== key for builtins).
    Role(RoleName),
    /// A single permission, by its [`crate::domain::value_objects::permission::Permission::key`].
    Permission(String),
}

impl GrantTarget {
    /// Persistence discriminant — the `target_kind` column value.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Role(_) => "role",
            Self::Permission(_) => "permission",
        }
    }

    /// The stored `target` column value — a role id or a permission key.
    #[must_use]
    pub fn value(&self) -> &str {
        match self {
            Self::Role(role) => role.as_str(),
            Self::Permission(key) => key.as_str(),
        }
    }
}

/// An explicit, scoped grant — "principal P holds TARGET within scope S", where
/// the target is a role or a single permission. A role grant materialises as a
/// linked Cedar template instance; a permission grant as a direct permit policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub id: String,
    pub principal: Principal,
    pub target: GrantTarget,
    pub scope: Scope,
}

impl Grant {
    /// A grant of a whole role.
    #[must_use]
    pub fn new(principal: Principal, role: RoleName, scope: Scope) -> Self {
        Self::with_target(principal, GrantTarget::Role(role), scope)
    }

    /// A grant of a single permission (by its `Permission::key()`), additive to
    /// the principal's role-derived permissions.
    #[must_use]
    pub fn with_permission(
        principal: Principal,
        permission: impl Into<String>,
        scope: Scope,
    ) -> Self {
        Self::with_target(principal, GrantTarget::Permission(permission.into()), scope)
    }

    #[must_use]
    fn with_target(principal: Principal, target: GrantTarget, scope: Scope) -> Self {
        Self {
            id: ulid::Ulid::new().to_string().to_lowercase(),
            principal,
            target,
            scope,
        }
    }
}

mod repository;
mod use_case;

pub use repository::*;
pub use use_case::*;
