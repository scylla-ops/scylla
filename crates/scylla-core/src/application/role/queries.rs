//! The role's reads. One block per query, in the order it runs: the struct, its permission, its
//! output type, what `Fetch` reads.

use super::RoleUseCases;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::permission::{PERMISSION_CATALOG, Permission};
use async_trait::async_trait;
use scylla_auth::authz::{
    EffectiveScope, GrantRepository, PolicyControl, Principal, Role, RoleRepository,
};
use scylla_extension::{Authorized, Describe, Fetch, Fetched, Query, Run};

#[derive(Debug)]
pub struct ListRoles;

impl Describe for ListRoles {
    fn permission(&self) -> Permission {
        Permission::ManageRoles
    }
}

impl Query for ListRoles {
    type Output = Vec<Role>;
}

#[async_trait]
impl<RR, GR, PC> Run<Fetch<ListRoles>> for RoleUseCases<RR, GR, PC>
where
    RR: RoleRepository,
    GR: GrantRepository,
    PC: PolicyControl,
{
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
    fn permission(&self) -> Permission {
        Permission::ManageRoles
    }
}

impl Query for GetRole {
    type Output = Role;
}

#[async_trait]
impl<RR, GR, PC> Run<Fetch<GetRole>> for RoleUseCases<RR, GR, PC>
where
    RR: RoleRepository,
    GR: GrantRepository,
    PC: PolicyControl,
{
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
    fn permission(&self) -> Permission {
        Permission::ManageRoles
    }
}

impl Query for ListAuthzVocabulary {
    type Output = &'static [(&'static str, &'static str)];
}

#[async_trait]
impl<RR, GR, PC> Run<Fetch<ListAuthzVocabulary>> for RoleUseCases<RR, GR, PC>
where
    RR: RoleRepository,
    GR: GrantRepository,
    PC: PolicyControl,
{
    async fn run(
        &self,
        input: Authorized<ListAuthzVocabulary>,
    ) -> DomainResult<Fetched<ListAuthzVocabulary>> {
        Ok(input.fetched(PERMISSION_CATALOG.as_slice()))
    }
}

/// Another principal's permissions, the admin view; a caller reads its own through
/// `RoleUseCases::my_permissions`.
#[derive(Debug)]
pub struct GetEffectivePermissions {
    pub principal: Principal,
}

impl Describe for GetEffectivePermissions {
    fn permission(&self) -> Permission {
        Permission::ManageSystemGrants
    }
}

impl Query for GetEffectivePermissions {
    type Output = Vec<EffectiveScope>;
}

#[async_trait]
impl<RR, GR, PC> Run<Fetch<GetEffectivePermissions>> for RoleUseCases<RR, GR, PC>
where
    RR: RoleRepository,
    GR: GrantRepository,
    PC: PolicyControl,
{
    async fn run(
        &self,
        input: Authorized<GetEffectivePermissions>,
    ) -> DomainResult<Fetched<GetEffectivePermissions>> {
        let scopes = self.effective_scopes(&input.command().principal).await?;
        Ok(input.fetched(scopes))
    }
}
