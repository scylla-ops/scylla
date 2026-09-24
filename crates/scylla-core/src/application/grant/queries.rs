//! The grant's reads. One block per query, in the order it runs: the struct, its permission, its
//! output type, what `Fetch` reads.

use super::{GrantUseCases, manage_permission};
use crate::domain::errors::DomainResult;
use crate::domain::permission::Permission;
use async_trait::async_trait;
use scylla_auth::authz::{Grant, Scope};
use scylla_extension::{Authorized, Describe, Fetch, Fetched, Query, Run};

/// Without a scope, every grant of the installation; with one, the grants bound at that scope.
#[derive(Debug)]
pub struct ListGrants {
    pub scope: Option<Scope>,
}

impl Describe for ListGrants {
    fn permission(&self) -> Permission {
        self.scope
            .as_ref()
            .map_or(Permission::ManageSystemGrants, manage_permission)
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
