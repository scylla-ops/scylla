use crate::application::authz::grant::ScopeKind;
use crate::domain::entities::OrganizationId;
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Sentinel permission meaning **full control** — any action within the grant's
/// scope. Builtin admin roles hold exactly this; it maps to the unconstrained
/// Cedar action body (the former full-control role template), so an admin role
/// automatically covers any permission added later without a re-seed.
pub const FULL_CONTROL: &str = "*";

/// A role: a named, editable bundle of permissions bound to a scope kind.
///
/// Builtin roles (`builtin`, `owner_org` = `None`) are global and seeded on
/// first boot; custom roles are owned by an organization (tenant-isolated). The
/// live Cedar policy set is generated from these — a grant of a role confers the
/// role's permissions within the grant's scope, so editing a role's permissions
/// changes authorization on the next reload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Role {
    /// Opaque id. For builtins it equals [`Role::key`] (a stable string such as
    /// `"organization-admin"`), which is also the value a grant references.
    pub id: String,
    /// Stable identifier for builtins; `None` for tenant custom roles.
    pub key: Option<String>,
    pub name: String,
    pub description: String,
    /// The scope kind a grant of this role must bind to.
    pub scope: ScopeKind,
    /// Owning organization for a tenant custom role; `None` = global (builtin).
    pub owner_org: Option<OrganizationId>,
    pub builtin: bool,
    /// Permission keys ([`crate::domain::value_objects::permission::Permission::key`]),
    /// or a single [`FULL_CONTROL`] entry.
    pub permissions: Vec<String>,
}

impl Role {
    /// Whether this role confers full control (any action) within its scope.
    #[must_use]
    pub fn is_full_control(&self) -> bool {
        self.permissions.iter().any(|p| p == FULL_CONTROL)
    }
}

/// Read access to role definitions. Read at `CedarPermissionService`
/// construction and on every reload to generate the per-role Cedar templates.
#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// Every role with its permission set — global builtins plus all tenant
    /// custom roles. Drives the Cedar policy-set generation.
    async fn list_all(&self) -> DomainResult<Vec<Role>>;
}
