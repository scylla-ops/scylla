use crate::application::authz::grant::{
    GrantRepository, GrantTarget, ORGANIZATION_ADMIN_ROLE, PROJECT_ADMIN_ROLE, ScopeKind,
};
use crate::application::authz::policy::PolicyControl;
use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::domain::entities::OrganizationId;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::{Permission, is_known_permission};
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

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

    /// A new global custom role (no owner org, not builtin) with a fresh id.
    #[must_use]
    pub fn new_custom(
        name: String,
        description: String,
        scope: ScopeKind,
        permissions: Vec<String>,
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string().to_lowercase(),
            key: None,
            name,
            description,
            scope,
            owner_org: None,
            builtin: false,
            permissions,
        }
    }
}

/// Validate a role's permission set: every entry must be a known permission key
/// (or the [`FULL_CONTROL`] sentinel), and the set must be non-empty (a role that
/// confers nothing is almost always a mistake).
pub fn validate_role_permissions(permissions: &[String]) -> DomainResult<()> {
    if permissions.is_empty() {
        return Err(DomainError::validation(
            "a role must have at least one permission",
        ));
    }
    for p in permissions {
        if p != FULL_CONTROL && !is_known_permission(p) {
            return Err(DomainError::validation(format!("unknown permission '{p}'")));
        }
    }
    Ok(())
}

/// Read + write access to role definitions. `list_all` is read at
/// `CedarPermissionService` construction and on every reload to generate the
/// per-role Cedar templates; the rest back `RoleUseCases`.
#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// Every role with its permission set — global builtins plus all tenant
    /// custom roles. Drives the Cedar policy-set generation.
    async fn list_all(&self) -> DomainResult<Vec<Role>>;
    /// One role by id (with its permission set), or `None` if absent.
    async fn get(&self, id: &str) -> DomainResult<Option<Role>>;
    /// Insert a new role and its permissions atomically.
    async fn create(&self, role: &Role) -> DomainResult<()>;
    /// Replace a role's name / description / permission set atomically.
    async fn update(&self, role: &Role) -> DomainResult<()>;
    /// Delete a role (its permissions cascade). May fail if the role is still
    /// pointed at by a default-role binding (`ON DELETE RESTRICT`).
    async fn delete(&self, id: &str) -> DomainResult<()>;
}

/// Admin management of the dynamic role catalog — every method is gated by
/// `Permission::ManageRoles` (system). A created/edited/deleted role is applied
/// live via [`PolicyControl::reload`] (its generated Cedar template is rebuilt),
/// so changes take effect without a control-plane restart. (First cut: global
/// roles only; org-owned custom roles + org-admin management come later.)
#[derive(Constructor)]
pub struct RoleUseCases<RR, GR, PS, PC>
where
    RR: RoleRepository,
    GR: GrantRepository,
    PS: PermissionService,
    PC: PolicyControl,
{
    role_repo: Arc<RR>,
    grant_repo: Arc<GR>,
    permission_service: Arc<PS>,
    policy_control: Arc<PC>,
}

impl<RR, GR, PS, PC> RoleUseCases<RR, GR, PS, PC>
where
    RR: RoleRepository,
    GR: GrantRepository,
    PS: PermissionService,
    PC: PolicyControl,
{
    #[instrument(skip(self, caller))]
    pub async fn list(&self, caller: &CallerContext) -> DomainResult<Vec<Role>> {
        self.permission_service
            .check(caller, Permission::ManageRoles)
            .await?;
        self.role_repo.list_all().await
    }

    #[instrument(skip(self, caller))]
    pub async fn get(&self, caller: &CallerContext, id: &str) -> DomainResult<Role> {
        self.permission_service
            .check(caller, Permission::ManageRoles)
            .await?;
        self.role_repo
            .get(id)
            .await?
            .ok_or_else(|| DomainError::not_found("role", id))
    }

    #[instrument(skip(self, caller))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        name: String,
        description: String,
        scope: ScopeKind,
        permissions: Vec<String>,
    ) -> DomainResult<Role> {
        self.permission_service
            .check(caller, Permission::ManageRoles)
            .await?;
        validate_role_permissions(&permissions)?;
        let role = Role::new_custom(name, description, scope, permissions);
        self.role_repo.create(&role).await?;
        self.policy_control.reload().await?;
        Ok(role)
    }

    #[instrument(skip(self, caller))]
    pub async fn update(
        &self,
        caller: &CallerContext,
        id: &str,
        name: String,
        description: String,
        permissions: Vec<String>,
    ) -> DomainResult<Role> {
        self.permission_service
            .check(caller, Permission::ManageRoles)
            .await?;
        validate_role_permissions(&permissions)?;
        let mut role = self
            .role_repo
            .get(id)
            .await?
            .ok_or_else(|| DomainError::not_found("role", id))?;
        // A role's scope kind and builtin status are immutable; only its name,
        // description and permission set can change.
        role.name = name;
        role.description = description;
        role.permissions = permissions;
        self.role_repo.update(&role).await?;
        self.policy_control.reload().await?;
        Ok(role)
    }

    #[instrument(skip(self, caller))]
    pub async fn delete(&self, caller: &CallerContext, id: &str) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::ManageRoles)
            .await?;
        let role = self
            .role_repo
            .get(id)
            .await?
            .ok_or_else(|| DomainError::not_found("role", id))?;
        if role.builtin {
            return Err(DomainError::business_rule(
                "builtin roles cannot be deleted",
            ));
        }
        // Refuse to orphan grants: a role still granted to someone must be
        // unassigned first, otherwise those grants would silently stop working.
        let grants = self.grant_repo.list_all().await?;
        if grants
            .iter()
            .any(|g| matches!(&g.target, GrantTarget::Role(r) if r.as_str() == id))
        {
            return Err(DomainError::business_rule(
                "role is still granted to one or more principals; revoke those grants first",
            ));
        }
        self.role_repo.delete(id).await?;
        self.policy_control.reload().await
    }
}

#[cfg(test)]
mod tests {
    use super::{FULL_CONTROL, validate_role_permissions};

    #[test]
    fn validate_role_permissions_accepts_keys_and_wildcard_rejects_others() {
        assert!(validate_role_permissions(&["runPipeline".into(), "readJob".into()]).is_ok());
        assert!(validate_role_permissions(&[FULL_CONTROL.into()]).is_ok());
        // Empty set and unknown keys are rejected.
        assert!(validate_role_permissions(&[]).is_err());
        assert!(validate_role_permissions(&["flyToTheMoon".into()]).is_err());
    }
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
