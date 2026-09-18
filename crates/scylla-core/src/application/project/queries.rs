//! The project's reads. One block per query, in the order it runs: the struct, its permission
//! and path, its output type, what `Fetch` reads.

use super::ProjectUseCases;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::application::{ProjectRepository, UserRepository};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, ProjectId, UserId};
use crate::domain::permission::Permission;
use crate::domain::project::Project;
use crate::domain::user::User;
use async_trait::async_trait;
use scylla_auth::authz::{PermissionService, PolicyControl, Visibility};
use scylla_extension::{Authorized, Describe, Fetch, Fetched, Query, Run};
use std::collections::HashMap;

#[derive(Debug)]
pub struct GetProject {
    pub id: ProjectId,
}

impl Describe for GetProject {
    fn permission(&self) -> Permission {
        Permission::ReadProject(self.id.clone())
    }
}

impl Query for GetProject {
    type Output = Project;
}

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

#[derive(Debug)]
pub struct ListProjects {
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListProjects {
    fn permission(&self) -> Permission {
        Permission::ListProjects
    }
}

impl Query for ListProjects {
    type Output = PaginatedResult<Project>;
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

/// Gated on `readOrganization`, not on `listProjectsByOrganization`: a project-only role must
/// see its own project, not be refused. The wider permission only widens the visible set.
#[derive(Debug)]
pub struct ListOrganizationProjects {
    pub organization_id: OrganizationId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListOrganizationProjects {
    fn permission(&self) -> Permission {
        Permission::ReadOrganization(self.organization_id.clone())
    }
}

impl Query for ListOrganizationProjects {
    type Output = PaginatedResult<Project>;
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

#[derive(Debug)]
pub struct ListProjectMembers {
    pub project_id: ProjectId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListProjectMembers {
    fn permission(&self) -> Permission {
        Permission::ListProjectMembers(self.project_id.clone())
    }
}

impl Query for ListProjectMembers {
    type Output = PaginatedResult<User>;
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
        Ok(input.fetched(PaginatedResult::from_parts(users, metadata)))
    }
}

#[derive(Debug)]
pub struct ListUserProjects {
    pub user_id: UserId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListUserProjects {
    fn permission(&self) -> Permission {
        Permission::ListUserProjects(self.user_id.clone())
    }
}

impl Query for ListUserProjects {
    type Output = PaginatedResult<Project>;
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
        let page = self
            .project_repo
            .list_for_user(&query.user_id, query.pagination.as_ref())
            .await?;
        Ok(input.fetched(page))
    }
}
