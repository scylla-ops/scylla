//! The role's reads. One block per query, in the order it runs: the struct, its access, its
//! output type, what `Fetch` reads.

use super::RoleUseCases;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::permission::{PERMISSION_CATALOG, Permission};
use async_trait::async_trait;
use scylla_auth::authz::{EffectiveScope, Principal, Role};
use scylla_extension::{Access, Authorized, Describe, Fetch, Fetched, Query, Run};

#[derive(Debug)]
pub struct ListRoles;

impl Describe for ListRoles {
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageRoles)
    }
}

impl Query for ListRoles {
    type Output = Vec<Role>;
}

#[async_trait]
impl Run<Fetch<ListRoles>> for RoleUseCases {
    async fn run(&self, input: Authorized<ListRoles>) -> DomainResult<Fetched<ListRoles>> {
        let roles = self.role_repo.list_all().await?;
        Ok(input.fetched(roles))
    }
}

#[derive(Debug)]
pub struct GetRole {
    pub id: String,
}

impl Describe for GetRole {
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageRoles)
    }
}

impl Query for GetRole {
    type Output = Role;
}

#[async_trait]
impl Run<Fetch<GetRole>> for RoleUseCases {
    async fn run(&self, input: Authorized<GetRole>) -> DomainResult<Fetched<GetRole>> {
        let id = &input.command().id;
        let role = self
            .role_repo
            .get(id)
            .await?
            .ok_or_else(|| DomainError::not_found("role", id))?;
        Ok(input.fetched(role))
    }
}

/// Every permission key with the resource type it targets.
#[derive(Debug)]
pub struct ListAuthzVocabulary;

impl Describe for ListAuthzVocabulary {
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageRoles)
    }
}

impl Query for ListAuthzVocabulary {
    type Output = &'static [(&'static str, &'static str)];
}

#[async_trait]
impl Run<Fetch<ListAuthzVocabulary>> for RoleUseCases {
    async fn run(
        &self,
        input: Authorized<ListAuthzVocabulary>,
    ) -> DomainResult<Fetched<ListAuthzVocabulary>> {
        Ok(input.fetched(PERMISSION_CATALOG.as_slice()))
    }
}

/// Another principal's permissions, the admin view; a caller reads its own through
/// `GetMyPermissions`.
#[derive(Debug)]
pub struct GetEffectivePermissions {
    pub principal: Principal,
}

impl Describe for GetEffectivePermissions {
    fn access(&self) -> Access {
        Access::Requires(Permission::ManageSystemGrants)
    }
}

impl Query for GetEffectivePermissions {
    type Output = Vec<EffectiveScope>;
}

#[async_trait]
impl Run<Fetch<GetEffectivePermissions>> for RoleUseCases {
    async fn run(
        &self,
        input: Authorized<GetEffectivePermissions>,
    ) -> DomainResult<Fetched<GetEffectivePermissions>> {
        let scopes = self.effective_scopes(&input.command().principal).await?;
        Ok(input.fetched(scopes))
    }
}

/// The caller's own permissions: no permission is asked. A service is refused here, not in
/// `Authorize`: it holds no grants, and an empty list would read as "no permissions".
#[derive(Debug)]
pub struct GetMyPermissions;

impl Describe for GetMyPermissions {
    fn access(&self) -> Access {
        Access::Authenticated
    }
}

impl Query for GetMyPermissions {
    type Output = Vec<EffectiveScope>;
}

#[async_trait]
impl Run<Fetch<GetMyPermissions>> for RoleUseCases {
    async fn run(
        &self,
        input: Authorized<GetMyPermissions>,
    ) -> DomainResult<Fetched<GetMyPermissions>> {
        let principal = Principal::from_caller(input.caller()).ok_or_else(|| {
            DomainError::forbidden("this caller is not a principal that holds grants")
        })?;
        let scopes = self.effective_scopes(&principal).await?;
        Ok(input.fetched(scopes))
    }
}
