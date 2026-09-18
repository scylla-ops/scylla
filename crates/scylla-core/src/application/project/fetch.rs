//! The use case's half of each query: read through the ports and return.

use super::queries::{
    GetProject, ListOrganizationProjects, ListProjectMembers, ListProjects, ListUserProjects,
};
use super::use_case::ProjectUseCases;
use crate::application::{ProjectRepository, UserRepository};
use crate::domain::errors::DomainResult;
use crate::domain::ids::ProjectId;
use crate::domain::permission::Permission;
use crate::domain::user::User;
use async_trait::async_trait;
use scylla_auth::authz::{PermissionService, PolicyControl, Visibility};
use scylla_extension::{Authorized, Fetch, Fetched, Run};
use std::collections::HashMap;

#[async_trait]
impl<P, U, PS, PC> Run<Fetch<GetProject>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<GetProject>) -> DomainResult<Fetched<GetProject>> {
        let project = self.project_repo.find_by_id(&input.command().id).await?;
        Ok(input.fetched(project))
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Fetch<ListProjects>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(&self, input: Authorized<ListProjects>) -> DomainResult<Fetched<ListProjects>> {
        let page = self
            .project_repo
            .list_all(input.command().pagination.as_ref())
            .await?;
        Ok(input.fetched(page))
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Fetch<ListOrganizationProjects>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    // The second check is a scoping decision, not a gate: it never refuses, it picks between
    // every project of the organization and the ones the caller's grants reach.
    async fn run(
        &self,
        input: Authorized<ListOrganizationProjects>,
    ) -> DomainResult<Fetched<ListOrganizationProjects>> {
        let query = input.command();
        let visible = if self
            .permission_service
            .check(
                input.caller(),
                Permission::ListProjectsByOrganization(query.organization_id.clone()),
            )
            .await
            .is_ok()
        {
            Visibility::All
        } else {
            self.visibility
                .visible_scopes(
                    input.caller(),
                    Permission::ReadProject(ProjectId::new("_")).key(),
                )
                .await?
        };
        let page = self
            .project_repo
            .list_by_organization(&query.organization_id, query.pagination.as_ref(), &visible)
            .await?;
        Ok(input.fetched(page))
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Fetch<ListProjectMembers>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(
        &self,
        input: Authorized<ListProjectMembers>,
    ) -> DomainResult<Fetched<ListProjectMembers>> {
        let query = input.command();
        let paginated = self
            .project_repo
            .list_principals(&query.project_id, query.pagination.as_ref())
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
        Ok(input.fetched((users, metadata)))
    }
}

#[async_trait]
impl<P, U, PS, PC> Run<Fetch<ListUserProjects>> for ProjectUseCases<P, U, PS, PC>
where
    P: ProjectRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    PS: PermissionService,
    PC: PolicyControl,
{
    async fn run(
        &self,
        input: Authorized<ListUserProjects>,
    ) -> DomainResult<Fetched<ListUserProjects>> {
        let query = input.command();
        let paginated = self
            .project_repo
            .list_for_user(&query.user_id, query.pagination.as_ref())
            .await?;
        Ok(input.fetched(paginated.into_parts()))
    }
}
