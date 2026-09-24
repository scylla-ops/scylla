use crate::authz::grant::{Scope, ScopeKind};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::OrganizationId;
use crate::domain::permission::permission_resource_type;
use async_trait::async_trait;

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
