//! The grant's writes. One block per command, in the order it runs: the struct, its access,
//! its payload types, what `Prepare` checks and builds, what `Persist` writes. The policy reload
//! sits next to the write that changes the grant set.

use super::GrantUseCases;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::GrantId;
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use async_trait::async_trait;
use scylla_auth::authz::{
    Grant, Principal, Scope, is_owner_role, removal_orphans_scope, validate_role_in_db,
};
use scylla_extension::{
    Access, Authorized, Command, Committed, Deleted, Describe, Draft, Persist, Prepare, Prepared,
    Run,
};

#[derive(Debug)]
pub struct CreateGrant {
    pub principal: Principal,
    pub role: RoleName,
    pub scope: Scope,
}

impl Describe for CreateGrant {
    fn access(&self) -> Access {
        Access::Requires(self.scope.manage_permission())
    }
}

impl Command for CreateGrant {
    type Staged = Draft<Grant>;
    type Committed = Grant;
}

#[async_trait]
impl Run<Prepare<CreateGrant>> for GrantUseCases {
    async fn run(&self, input: Authorized<CreateGrant>) -> DomainResult<Prepared<CreateGrant>> {
        let cmd = input.command();
        let grant = Grant::new(cmd.principal.clone(), cmd.role.clone(), cmd.scope.clone());
        // A stored grant must always be emittable into Cedar.
        validate_role_in_db(&*self.role_repo, &grant.role, &grant.scope).await?;
        self.require_grantee_in_organization(&grant).await?;
        self.check_no_escalation(input.caller(), &grant).await?;
        Ok(input.prepared(Draft::new(grant)))
    }
}

#[async_trait]
impl Run<Persist<CreateGrant>> for GrantUseCases {
    async fn run(&self, input: Prepared<CreateGrant>) -> DomainResult<Committed<CreateGrant>> {
        input
            .commit(async |draft| {
                let grant = draft.into_inner();
                self.grant_repo.create(&grant).await?;
                self.policy_control.reload().await?;
                Ok(grant)
            })
            .await
    }
}

#[derive(Debug)]
pub struct RevokeAllAccess {
    pub principal: Principal,
    pub scope: Scope,
}

impl Describe for RevokeAllAccess {
    fn access(&self) -> Access {
        Access::Requires(self.scope.manage_permission())
    }
}

/// Stages the principal alone: the rows go in one statement, by principal and scope. `Committed`
/// is the number of grants removed.
impl Command for RevokeAllAccess {
    type Staged = Principal;
    type Committed = u64;
}

#[async_trait]
impl Run<Prepare<RevokeAllAccess>> for GrantUseCases {
    async fn run(
        &self,
        input: Authorized<RevokeAllAccess>,
    ) -> DomainResult<Prepared<RevokeAllAccess>> {
        let cmd = input.command();
        let grants = self.grant_repo.list_all().await?;
        if removal_orphans_scope(&grants, &cmd.scope, &cmd.principal) {
            return Err(DomainError::business_rule(
                "cannot remove the last owner of this scope",
            ));
        }
        let principal = cmd.principal.clone();
        Ok(input.prepared(principal))
    }
}

#[async_trait]
impl Run<Persist<RevokeAllAccess>> for GrantUseCases {
    async fn run(
        &self,
        input: Prepared<RevokeAllAccess>,
    ) -> DomainResult<Committed<RevokeAllAccess>> {
        let scope = input.command().scope.clone();
        input
            .commit(async |principal| {
                let removed = self.grant_repo.revoke_all(&principal, &scope).await?;
                self.policy_control.reload().await?;
                if let Principal::App(app_id) = &principal {
                    self.agent_registry.disconnect(app_id);
                }
                Ok(removed)
            })
            .await
    }
}

#[derive(Debug)]
pub struct RevokeGrant {
    pub id: GrantId,
}

impl Describe for RevokeGrant {
    fn access(&self) -> Access {
        Access::Requires(Permission::RevokeGrant(self.id.clone()))
    }
}

/// An unknown id stages `None` and commits nothing: the call succeeds without change.
impl Command for RevokeGrant {
    type Staged = Option<Grant>;
    type Committed = Option<Deleted<Grant>>;
}

#[async_trait]
impl Run<Prepare<RevokeGrant>> for GrantUseCases {
    async fn run(&self, input: Authorized<RevokeGrant>) -> DomainResult<Prepared<RevokeGrant>> {
        let grants = self.grant_repo.list_all().await?;
        let id = input.command().id.as_str();
        let grant = grants.iter().find(|g| g.id == id).cloned();

        // Only a User owner grant is guarded; an App never counts as the retained owner.
        if let Some(g) = &grant
            && is_owner_role(&g.role)
            && matches!(g.principal, Principal::User(_))
            && !grants.iter().any(|o| {
                o.id != g.id
                    && o.role == g.role
                    && o.scope == g.scope
                    && matches!(o.principal, Principal::User(_))
            })
        {
            return Err(DomainError::business_rule(
                "cannot revoke the last owner of this scope",
            ));
        }
        Ok(input.prepared(grant))
    }
}

#[async_trait]
impl Run<Persist<RevokeGrant>> for GrantUseCases {
    async fn run(&self, input: Prepared<RevokeGrant>) -> DomainResult<Committed<RevokeGrant>> {
        input
            .commit(async |grant| {
                let Some(grant) = grant else {
                    return Ok(None);
                };
                self.grant_repo.delete(&grant.id).await?;
                self.policy_control.reload().await?;
                if let Principal::App(app_id) = &grant.principal {
                    self.agent_registry.disconnect(app_id);
                }
                Ok(Some(Deleted::new(grant)))
            })
            .await
    }
}
