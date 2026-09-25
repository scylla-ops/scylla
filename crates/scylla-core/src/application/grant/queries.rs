//! The grant's reads. One block per query, in the order it runs: the struct, its access, its
//! output type, what `Fetch` reads.

use super::GrantUseCases;
use crate::domain::errors::DomainResult;
use crate::domain::permission::Permission;
use async_trait::async_trait;
use scylla_auth::authz::{Grant, GrantableRole, Scope, ScopeKind, grantable_roles};
use scylla_extension::{Access, Authorized, Describe, Fetch, Fetched, Query, Run};

/// Without a scope, every grant of the installation; with one, the grants bound at that scope.
#[derive(Debug)]
pub struct ListGrants {
    pub scope: Option<Scope>,
}

impl Describe for ListGrants {
    fn access(&self) -> Access {
        Access::Requires(
            self.scope
                .as_ref()
                .map_or(Permission::ManageSystemGrants, Scope::manage_permission),
        )
    }
}

impl Query for ListGrants {
    type Output = Vec<Grant>;
}

#[async_trait]
impl Run<Fetch<ListGrants>> for GrantUseCases {
    async fn run(&self, input: Authorized<ListGrants>) -> DomainResult<Fetched<ListGrants>> {
        let grants = self.grant_repo.list_all().await?;
        let grants = match &input.command().scope {
            Some(scope) => grants.into_iter().filter(|g| &g.scope == scope).collect(),
            None => grants,
        };
        Ok(input.fetched(grants))
    }
}

/// The roles a grant may bind, at one scope kind or at all. The catalog is static data: no
/// permission is asked.
#[derive(Debug)]
pub struct ListGrantableRoles {
    pub scope_kind: Option<ScopeKind>,
}

impl Describe for ListGrantableRoles {
    fn access(&self) -> Access {
        Access::Authenticated
    }
}

impl Query for ListGrantableRoles {
    type Output = Vec<GrantableRole>;
}

#[async_trait]
impl Run<Fetch<ListGrantableRoles>> for GrantUseCases {
    async fn run(
        &self,
        input: Authorized<ListGrantableRoles>,
    ) -> DomainResult<Fetched<ListGrantableRoles>> {
        let roles = grantable_roles(input.command().scope_kind);
        Ok(input.fetched(roles))
    }
}
