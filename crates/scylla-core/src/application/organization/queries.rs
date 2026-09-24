//! The organization's reads. One block per query, in the order it runs: the struct, its
//! permission, its output type, what `Fetch` reads.

use super::OrganizationUseCases;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, UserId};
use crate::domain::organization::Organization;
use crate::domain::permission::Permission;
use crate::domain::user::User;
use async_trait::async_trait;
use scylla_extension::{Authorized, Describe, Fetch, Fetched, Query, Run};
use std::collections::HashMap;

#[derive(Debug)]
pub struct GetOrganization {
    pub id: OrganizationId,
}

impl Describe for GetOrganization {
    fn permission(&self) -> Permission {
        Permission::ReadOrganization(self.id.clone())
    }
}

impl Query for GetOrganization {
    type Output = Organization;
}

#[async_trait]
impl Run<Fetch<GetOrganization>> for OrganizationUseCases {
    async fn run(
        &self,
        input: Authorized<GetOrganization>,
    ) -> DomainResult<Fetched<GetOrganization>> {
        let organization = self.org_repo.find_by_id(&input.command().id).await?;
        Ok(input.fetched(organization))
    }
}

#[derive(Debug)]
pub struct ListOrganizations {
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListOrganizations {
    fn permission(&self) -> Permission {
        Permission::ListOrganizations
    }
}

impl Query for ListOrganizations {
    type Output = PaginatedResult<Organization>;
}

#[async_trait]
impl Run<Fetch<ListOrganizations>> for OrganizationUseCases {
    async fn run(
        &self,
        input: Authorized<ListOrganizations>,
    ) -> DomainResult<Fetched<ListOrganizations>> {
        let page = self
            .org_repo
            .list_all(input.command().pagination.as_ref())
            .await?;
        Ok(input.fetched(page))
    }
}

#[derive(Debug)]
pub struct ListOrganizationMembers {
    pub organization_id: OrganizationId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListOrganizationMembers {
    fn permission(&self) -> Permission {
        Permission::ListOrganizationMembers(self.organization_id.clone())
    }
}

impl Query for ListOrganizationMembers {
    type Output = PaginatedResult<User>;
}

#[async_trait]
impl Run<Fetch<ListOrganizationMembers>> for OrganizationUseCases {
    async fn run(
        &self,
        input: Authorized<ListOrganizationMembers>,
    ) -> DomainResult<Fetched<ListOrganizationMembers>> {
        let query = input.command();
        let paginated = self
            .org_repo
            .list_principals(&query.organization_id, query.pagination.as_ref())
            .await?;
        let (user_ids, metadata) = paginated.into_parts();
        let mut by_id: HashMap<String, User> = self
            .user_repo
            .find_by_ids(&user_ids)
            .await?
            .into_iter()
            .map(|u| (u.id().as_str().to_owned(), u))
            .collect();
        let users = user_ids
            .iter()
            .filter_map(|id| by_id.remove(id.as_str()))
            .collect();
        Ok(input.fetched(PaginatedResult::from_parts(users, metadata)))
    }
}

#[derive(Debug)]
pub struct ListUserOrganizations {
    pub user_id: UserId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListUserOrganizations {
    fn permission(&self) -> Permission {
        Permission::ListUserOrganizations(self.user_id.clone())
    }
}

impl Query for ListUserOrganizations {
    type Output = PaginatedResult<Organization>;
}

#[async_trait]
impl Run<Fetch<ListUserOrganizations>> for OrganizationUseCases {
    async fn run(
        &self,
        input: Authorized<ListUserOrganizations>,
    ) -> DomainResult<Fetched<ListUserOrganizations>> {
        let query = input.command();
        let page = self
            .org_repo
            .list_for_user(&query.user_id, query.pagination.as_ref())
            .await?;
        Ok(input.fetched(page))
    }
}
