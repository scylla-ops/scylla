use crate::application::authz::grant::{ORGANIZATION_ADMIN_ROLE, PROJECT_ADMIN_ROLE, ScopeKind};
use crate::domain::entities::OrganizationId;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::role::name::RoleName;
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

/// A "default role" slot: a named pointer the creation flows resolve to decide
/// which role to grant the creator, decoupling them from a hard-coded role name.
/// An admin can rebind a slot to a custom role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultRoleSlot {
    /// Role granted to the creator of an organization.
    OrgCreation,
    /// Role granted to the creator of a project.
    ProjectCreation,
}

impl DefaultRoleSlot {
    /// The `default_role_bindings.slot` column value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OrgCreation => "org_creation",
            Self::ProjectCreation => "project_creation",
        }
    }

    /// The builtin role key this slot falls back to when no binding is set.
    #[must_use]
    pub fn builtin_role(self) -> &'static str {
        match self {
            Self::OrgCreation => ORGANIZATION_ADMIN_ROLE,
            Self::ProjectCreation => PROJECT_ADMIN_ROLE,
        }
    }
}

/// Read access to the configurable default-role pointers (`default_role_bindings`).
#[async_trait]
pub trait DefaultRoleBindingRepository: Send + Sync {
    /// The role bound to `slot`, if the binding exists and its role still exists;
    /// `None` lets the caller fall back to the slot's builtin role.
    async fn role_for_slot(&self, slot: DefaultRoleSlot) -> DomainResult<Option<RoleName>>;
}

/// Resolve the role a default slot points to, falling back to the slot's builtin
/// role when no binding is set. The creation flows call this instead of naming a
/// role directly, so the assigned role stays configurable.
pub async fn resolve_default_role(
    bindings: &dyn DefaultRoleBindingRepository,
    slot: DefaultRoleSlot,
) -> DomainResult<RoleName> {
    match bindings.role_for_slot(slot).await? {
        Some(role) => Ok(role),
        None => RoleName::new(slot.builtin_role()),
    }
}
