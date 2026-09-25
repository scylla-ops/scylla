pub mod commands;
pub mod queries;

pub use commands::{CreateGrant, RevokeAllAccess, RevokeGrant};
pub use queries::{ListGrantableRoles, ListGrants};

use crate::application::agent::dispatch_port::AgentDispatch;
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::permission::ResourceRef;
use crate::domain::role::RoleName;
use derive_more::Constructor;
use scylla_auth::authz::{
    AuthzEntityProvider, FULL_CONTROL, Grant, GrantRepository, PolicyControl, Principal,
    RoleRepository, Scope, permissions_by_role,
};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The grant aggregate's stage runners, one block per action in `commands.rs` and `queries.rs`.
/// `Actions::run` drives it; the methods below are the checks of `CreateGrant`.
#[derive(Constructor)]
pub struct GrantUseCases {
    pub(super) grant_repo: Arc<dyn GrantRepository>,
    pub(super) role_repo: Arc<dyn RoleRepository>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
    pub(super) agent_registry: Arc<dyn AgentDispatch>,
    pub(super) entity_provider: Arc<dyn AuthzEntityProvider>,
}

enum Holding {
    Full,
    Keys(BTreeSet<String>),
}

impl GrantUseCases {
    /// The tenant boundary: without it a project admin could attach any account in the installation.
    /// Org and System grants are the admission itself; Apps are owned by their org by construction.
    pub(super) async fn require_grantee_in_organization(&self, grant: &Grant) -> DomainResult<()> {
        let Principal::User(_) = &grant.principal else {
            return Ok(());
        };
        let Scope::Project(project_id) = &grant.scope else {
            return Ok(());
        };
        let org_id = self
            .entity_provider
            .resource_ancestors(&ResourceRef::Project(project_id.clone()))
            .await?
            .organization
            .ok_or_else(|| DomainError::not_found("Project", project_id.to_string()))?;

        let admitted = self.grant_repo.list_all().await?.into_iter().any(|g| {
            g.principal == grant.principal
                && matches!(&g.scope, Scope::Organization(o) if o == &org_id)
        });

        if admitted {
            Ok(())
        } else {
            Err(DomainError::business_rule(
                "the user must already have access to this organization before \
                 receiving a grant on one of its projects",
            ))
        }
    }

    /// A delegator may only confer what it holds at the scope; otherwise `manageOrgGrants` alone could grant `organization-admin`.
    /// Project scope is not subset-checked yet; it stays gated by `manageProjectGrants`.
    pub(super) async fn check_no_escalation(
        &self,
        caller: &CallerContext,
        grant: &Grant,
    ) -> DomainResult<()> {
        let Some(principal) = Principal::from_caller(caller) else {
            return Ok(());
        };
        if matches!(grant.scope, Scope::Project(_)) {
            return Ok(());
        }

        let allowed = match (
            self.holding_at(&principal, &grant.scope).await?,
            self.role_keys(&grant.role).await?,
        ) {
            (Holding::Full, _) => true,
            (Holding::Keys(_), None) => false,
            (Holding::Keys(have), Some(want)) => want.iter().all(|k| have.contains(k)),
        };
        if allowed {
            Ok(())
        } else {
            Err(DomainError::business_rule(
                "cannot grant permissions you do not hold at this scope (no privilege escalation)",
            ))
        }
    }

    async fn role_keys(&self, role: &RoleName) -> DomainResult<Option<BTreeSet<String>>> {
        match self.role_repo.get(role.as_str()).await? {
            Some(r) if r.is_full_control() => Ok(None),
            Some(r) => Ok(Some(r.permissions.into_iter().collect())),
            None => Ok(Some(BTreeSet::new())),
        }
    }

    async fn holding_at(&self, principal: &Principal, scope: &Scope) -> DomainResult<Holding> {
        let role_perms = permissions_by_role(self.role_repo.as_ref()).await?;
        let grants = self.grant_repo.list_all().await?;

        let mut keys = BTreeSet::new();
        for g in grants.iter().filter(|g| g.principal == *principal) {
            if !(matches!(g.scope, Scope::System) || &g.scope == scope) {
                continue;
            }
            let perms: Vec<String> = role_perms.get(g.role.as_str()).cloned().unwrap_or_default();
            if perms.iter().any(|p| p == FULL_CONTROL) {
                return Ok(Holding::Full);
            }
            keys.extend(perms);
        }
        Ok(Holding::Keys(keys))
    }
}

#[cfg(test)]
mod tests;
