use crate::authz::grant::{GrantRepository, Principal, Scope, ScopeKind};
use crate::authz::policy::PolicyControl;
use crate::authz::service::PermissionService;
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::OrganizationId;
use crate::domain::permission::{PERMISSION_CATALOG, Permission, permission_resource_type};
use async_trait::async_trait;
use derive_more::Constructor;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use tracing::instrument;

/// Unconstrained Cedar action: an admin role covers permissions added later without a re-seed.
pub const FULL_CONTROL: &str = "*";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Role {
    pub id: String,
    pub key: Option<String>,
    pub name: String,
    pub description: String,
    pub scope: ScopeKind,
    pub owner_org: Option<OrganizationId>,
    pub builtin: bool,
    pub permissions: Vec<String>,
}

impl Role {
    #[must_use]
    pub fn is_full_control(&self) -> bool {
        self.permissions.iter().any(|p| p == FULL_CONTROL)
    }

    #[must_use]
    pub fn new_custom(
        name: String,
        description: String,
        scope: ScopeKind,
        permissions: Vec<String>,
    ) -> Self {
        Self {
            id: scylla_domain::domain::ids::new_id(),
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

pub fn resource_home_scope(resource_type: &str) -> ScopeKind {
    match resource_type {
        "organization" | "app" => ScopeKind::Organization,
        "project" | "pipeline" | "job" => ScopeKind::Project,
        _ => ScopeKind::System,
    }
}

pub fn validate_role_permissions(permissions: &[String], scope: ScopeKind) -> DomainResult<()> {
    if permissions.is_empty() {
        return Err(DomainError::validation(
            "a role must have at least one permission",
        ));
    }
    for p in permissions {
        if p == FULL_CONTROL {
            continue;
        }
        let Some(resource_type) = permission_resource_type(p) else {
            return Err(DomainError::validation(format!("unknown permission '{p}'")));
        };
        let home = resource_home_scope(resource_type);
        if !scope.covers(home) {
            return Err(DomainError::validation(format!(
                "permission '{p}' targets a {resource_type} and is not usable in a {} role; \
                 grant it in a role scoped at {} or broader",
                scope.as_str(),
                home.as_str()
            )));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveScope {
    pub scope: Scope,
    pub full_control: bool,
    pub permissions: Vec<String>,
}

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn list_all(&self) -> DomainResult<Vec<Role>>;
    async fn get(&self, id: &str) -> DomainResult<Option<Role>>;
    async fn create(&self, role: &Role) -> DomainResult<()>;
    async fn update(&self, role: &Role) -> DomainResult<()>;
    async fn delete(&self, id: &str) -> DomainResult<()>;
}

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
    pub async fn authz_vocabulary(
        &self,
        caller: &CallerContext,
    ) -> DomainResult<&'static [(&'static str, &'static str)]> {
        self.permission_service
            .check(caller, Permission::ManageRoles)
            .await?;
        Ok(PERMISSION_CATALOG.as_slice())
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

    #[instrument(skip_all, fields(name = %name, scope = scope.as_str(), permissions = permissions.len()))]
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
        validate_role_permissions(&permissions, scope)?;
        let role = Role::new_custom(name, description, scope, permissions);
        self.role_repo.create(&role).await?;
        self.policy_control.reload().await?;
        Ok(role)
    }

    #[instrument(skip_all, fields(role_id = %id, name = %name, permissions = permissions.len()))]
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
        let mut role = self
            .role_repo
            .get(id)
            .await?
            .ok_or_else(|| DomainError::not_found("role", id))?;
        validate_role_permissions(&permissions, role.scope)?;
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
        // A role still granted must be unassigned first; those grants would silently stop working.
        let grants = self.grant_repo.list_all().await?;
        if grants.iter().any(|g| g.role.as_str() == id) {
            return Err(DomainError::business_rule(
                "role is still granted to one or more principals; revoke those grants first",
            ));
        }
        self.role_repo.delete(id).await?;
        self.policy_control.reload().await
    }

    #[instrument(skip_all, fields(principal = %principal))]
    pub async fn effective_permissions(
        &self,
        caller: &CallerContext,
        principal: &Principal,
    ) -> DomainResult<Vec<EffectiveScope>> {
        self.permission_service
            .check(caller, Permission::ManageSystemGrants)
            .await?;
        self.resolve_effective_permissions(principal).await
    }

    /// Service and Anonymous are refused: an empty list would read as "no permissions".
    #[instrument(skip(self, caller))]
    pub async fn my_permissions(
        &self,
        caller: &CallerContext,
    ) -> DomainResult<Vec<EffectiveScope>> {
        let principal = Principal::from_caller(caller).ok_or_else(|| {
            DomainError::Forbidden("this caller is not a principal that holds grants".to_string())
        })?;
        self.resolve_effective_permissions(&principal).await
    }

    /// Grants as bound per scope: a System grant is not re-listed under every org and project.
    async fn resolve_effective_permissions(
        &self,
        principal: &Principal,
    ) -> DomainResult<Vec<EffectiveScope>> {
        let role_perms: HashMap<String, Vec<String>> = self
            .role_repo
            .list_all()
            .await?
            .into_iter()
            .map(|r| (r.id, r.permissions))
            .collect();
        let grants = self.grant_repo.list_all().await?;

        let mut groups: Vec<(Scope, BTreeSet<String>)> = Vec::new();
        for grant in grants.iter().filter(|g| g.principal == *principal) {
            let perms: Vec<String> = role_perms
                .get(grant.role.as_str())
                .cloned()
                .unwrap_or_default();
            if let Some(idx) = groups.iter().position(|(s, _)| *s == grant.scope) {
                groups[idx].1.extend(perms);
            } else {
                groups.push((grant.scope.clone(), perms.into_iter().collect()));
            }
        }

        Ok(groups
            .into_iter()
            .map(|(scope, set)| {
                let full_control = set.contains(FULL_CONTROL);
                EffectiveScope {
                    scope,
                    full_control,
                    permissions: if full_control {
                        Vec::new()
                    } else {
                        set.into_iter().collect()
                    },
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{FULL_CONTROL, ScopeKind, validate_role_permissions};

    #[test]
    fn validate_role_permissions_accepts_keys_and_wildcard_rejects_others() {
        assert!(
            validate_role_permissions(&["runPipeline".into(), "readJob".into()], ScopeKind::System)
                .is_ok()
        );
        assert!(validate_role_permissions(&[FULL_CONTROL.into()], ScopeKind::Project).is_ok());
        assert!(validate_role_permissions(&[], ScopeKind::System).is_err());
        assert!(validate_role_permissions(&["flyToTheMoon".into()], ScopeKind::System).is_err());
    }

    #[test]
    fn validate_role_permissions_enforces_scope_coherence() {
        assert!(
            validate_role_permissions(&["createOrganization".into()], ScopeKind::System).is_ok()
        );
        assert!(
            validate_role_permissions(&["createOrganization".into()], ScopeKind::Organization)
                .is_err()
        );
        assert!(
            validate_role_permissions(&["createOrganization".into()], ScopeKind::Project).is_err()
        );

        assert!(
            validate_role_permissions(&["createProject".into()], ScopeKind::Organization).is_ok()
        );
        assert!(validate_role_permissions(&["createProject".into()], ScopeKind::Project).is_err());

        assert!(validate_role_permissions(&["runPipeline".into()], ScopeKind::Project).is_ok());
        assert!(
            validate_role_permissions(&["runPipeline".into()], ScopeKind::Organization).is_ok()
        );
        assert!(validate_role_permissions(&["runPipeline".into()], ScopeKind::System).is_ok());
    }
}
