pub mod commands;
pub mod queries;

pub use commands::{CreateGrant, RevokeAllAccess};
pub use queries::ListGrants;

use crate::application::agent::dispatch_port::AgentDispatch;
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::permission::Permission;
use derive_more::Constructor;
use scylla_auth::authz::{
    AuthzEntityProvider, GrantRepository, PermissionService, PolicyControl, Principal,
    RoleRepository, Scope, is_owner_role,
};
use std::sync::Arc;
use tracing::instrument;

/// The grant aggregate's stage runners, one block per action in `commands.rs` and `queries.rs`.
/// `revoke` stays outside the pipeline: its permission is on the grant's scope, which only the
/// loaded grant knows, and `Describe` sees the command alone.
#[derive(Constructor)]
pub struct GrantUseCases<G: GrantRepository, PC: PolicyControl, PS: PermissionService> {
    pub(super) grant_repo: Arc<G>,
    pub(super) role_repo: Arc<dyn RoleRepository>,
    pub(super) policy_control: Arc<PC>,
    pub(super) permission_service: Arc<PS>,
    pub(super) agent_registry: Arc<dyn AgentDispatch>,
    pub(super) entity_provider: Arc<dyn AuthzEntityProvider>,
}

/// Cedar's `resource in ?resource` confines the caller to its own subtree; no trust in the caller.
pub(super) fn manage_permission(scope: &Scope) -> Permission {
    match scope {
        Scope::System => Permission::ManageSystemGrants,
        Scope::Organization(id) => Permission::ManageOrgGrants(id.clone()),
        Scope::Project(id) => Permission::ManageProjectGrants(id.clone()),
    }
}

impl<G: GrantRepository, PC: PolicyControl, PS: PermissionService> GrantUseCases<G, PC, PS> {
    #[instrument(skip(self, caller))]
    pub async fn revoke(&self, caller: &CallerContext, id: &str) -> DomainResult<()> {
        let grants = self.grant_repo.list_all().await?;
        let grant = grants.iter().find(|g| g.id == id).cloned();

        // Unknown id falls back to the system permission so only admins can probe ids.
        let perm = grant.as_ref().map_or(Permission::ManageSystemGrants, |g| {
            manage_permission(&g.scope)
        });
        self.permission_service.check(caller, perm).await?;

        // Only a User owner grant is guarded; an App never counts as the retained owner.
        if let Some(g) = &grant
            && is_owner_role(&g.role)
            && matches!(g.principal, Principal::User(_))
        {
            let other_human_owners = grants
                .iter()
                .filter(|o| {
                    o.id != g.id
                        && o.role == g.role
                        && o.scope == g.scope
                        && matches!(o.principal, Principal::User(_))
                })
                .count();
            if other_human_owners == 0 {
                return Err(DomainError::business_rule(
                    "cannot revoke the last owner of this scope",
                ));
            }
        }

        self.grant_repo.delete(id).await?;
        self.policy_control.reload().await?;

        if let Some(Principal::App(app_id)) = grant.map(|g| g.principal) {
            self.agent_registry.disconnect(&app_id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
