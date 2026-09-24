//! The grant's writes. One block per command, in the order it runs: the struct, its permission,
//! its payload types, what `Prepare` checks and builds, what `Persist` writes. The policy reload
//! sits next to the write that changes the grant set.

use super::{GrantUseCases, manage_permission};
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::permission::{Permission, ResourceRef};
use crate::domain::role::RoleName;
use async_trait::async_trait;
use scylla_auth::authz::{
    FULL_CONTROL, Grant, GrantRepository, PermissionService, PolicyControl, Principal, Scope,
    removal_orphans_scope, validate_role_in_db,
};
use scylla_extension::{
    Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug)]
pub struct CreateGrant {
    pub principal: Principal,
    pub role: RoleName,
    pub scope: Scope,
}

impl Describe for CreateGrant {
    fn permission(&self) -> Permission {
        manage_permission(&self.scope)
    }
}

impl Command for CreateGrant {
    type Staged = Draft<Grant>;
    type Committed = Grant;
}

#[async_trait]
impl<G, PC, PS> Run<Prepare<CreateGrant>> for GrantUseCases<G, PC, PS>
where
    G: GrantRepository,
    PC: PolicyControl,
    PS: PermissionService,
{
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
impl<G, PC, PS> Run<Persist<CreateGrant>> for GrantUseCases<G, PC, PS>
where
    G: GrantRepository,
    PC: PolicyControl,
    PS: PermissionService,
{
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

enum Holding {
    Full,
    Keys(BTreeSet<String>),
}

impl<G: GrantRepository, PC: PolicyControl, PS: PermissionService> GrantUseCases<G, PC, PS> {
    /// The tenant boundary: without it a project admin could attach any account in the installation.
    /// Org and System grants are the admission itself; Apps are owned by their org by construction.
    async fn require_grantee_in_organization(&self, grant: &Grant) -> DomainResult<()> {
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
    async fn check_no_escalation(&self, caller: &CallerContext, grant: &Grant) -> DomainResult<()> {
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
        let role_perms: HashMap<String, Vec<String>> = self
            .role_repo
            .list_all()
            .await?
            .into_iter()
            .map(|r| (r.id, r.permissions))
            .collect();
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

#[derive(Debug)]
pub struct RevokeAllAccess {
    pub principal: Principal,
    pub scope: Scope,
}

impl Describe for RevokeAllAccess {
    fn permission(&self) -> Permission {
        manage_permission(&self.scope)
    }
}

/// Stages the principal alone: the rows go in one statement, by principal and scope. `Committed`
/// is the number of grants removed.
impl Command for RevokeAllAccess {
    type Staged = Principal;
    type Committed = u64;
}

#[async_trait]
impl<G, PC, PS> Run<Prepare<RevokeAllAccess>> for GrantUseCases<G, PC, PS>
where
    G: GrantRepository,
    PC: PolicyControl,
    PS: PermissionService,
{
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
impl<G, PC, PS> Run<Persist<RevokeAllAccess>> for GrantUseCases<G, PC, PS>
where
    G: GrantRepository,
    PC: PolicyControl,
    PS: PermissionService,
{
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
