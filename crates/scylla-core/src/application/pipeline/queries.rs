//! The pipeline's reads. One block per query, in the order it runs: the struct, its permission,
//! its output type, what `Fetch` reads.

use super::PipelineUseCases;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::application::{JobRepository, PipelineRepository, ProjectRepository};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, PipelineId, ProjectId};
use crate::domain::permission::Permission;
use crate::domain::pipeline::Pipeline;
use async_trait::async_trait;
use scylla_auth::authz::PermissionService;
use scylla_extension::{Authorized, Describe, Fetch, Fetched, Query, Run};

#[derive(Debug)]
pub struct GetPipeline {
    pub id: PipelineId,
}

impl Describe for GetPipeline {
    fn permission(&self) -> Permission {
        Permission::ReadPipeline(self.id.clone())
    }
}

impl Query for GetPipeline {
    type Output = Pipeline;
}

#[async_trait]
impl<P, PR, J, PS> Run<Fetch<GetPipeline>> for PipelineUseCases<P, PR, J, PS>
where
    P: PipelineRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    J: JobRepository + Send + Sync,
    PS: PermissionService,
{
    async fn run(&self, input: Authorized<GetPipeline>) -> DomainResult<Fetched<GetPipeline>> {
        let pipeline = self.pipeline_repo.find_by_id(&input.command().id).await?;
        Ok(input.fetched(pipeline))
    }
}

#[derive(Debug)]
pub struct ListPipelines {
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListPipelines {
    fn permission(&self) -> Permission {
        Permission::ListPipelines
    }
}

impl Query for ListPipelines {
    type Output = PaginatedResult<Pipeline>;
}

#[async_trait]
impl<P, PR, J, PS> Run<Fetch<ListPipelines>> for PipelineUseCases<P, PR, J, PS>
where
    P: PipelineRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    J: JobRepository + Send + Sync,
    PS: PermissionService,
{
    async fn run(&self, input: Authorized<ListPipelines>) -> DomainResult<Fetched<ListPipelines>> {
        let page = self
            .pipeline_repo
            .list_all(input.command().pagination.as_ref())
            .await?;
        Ok(input.fetched(page))
    }
}

#[derive(Debug)]
pub struct ListProjectPipelines {
    pub project_id: ProjectId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListProjectPipelines {
    fn permission(&self) -> Permission {
        Permission::ListPipelinesByProject(self.project_id.clone())
    }
}

impl Query for ListProjectPipelines {
    type Output = PaginatedResult<Pipeline>;
}

#[async_trait]
impl<P, PR, J, PS> Run<Fetch<ListProjectPipelines>> for PipelineUseCases<P, PR, J, PS>
where
    P: PipelineRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    J: JobRepository + Send + Sync,
    PS: PermissionService,
{
    async fn run(
        &self,
        input: Authorized<ListProjectPipelines>,
    ) -> DomainResult<Fetched<ListProjectPipelines>> {
        let query = input.command();
        let page = self
            .pipeline_repo
            .list_by_project(&query.project_id, query.pagination.as_ref())
            .await?;
        Ok(input.fetched(page))
    }
}

#[derive(Debug)]
pub struct ListOrganizationPipelines {
    pub organization_id: OrganizationId,
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListOrganizationPipelines {
    fn permission(&self) -> Permission {
        Permission::ListPipelinesByOrganization(self.organization_id.clone())
    }
}

impl Query for ListOrganizationPipelines {
    type Output = PaginatedResult<Pipeline>;
}

#[async_trait]
impl<P, PR, J, PS> Run<Fetch<ListOrganizationPipelines>> for PipelineUseCases<P, PR, J, PS>
where
    P: PipelineRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    J: JobRepository + Send + Sync,
    PS: PermissionService,
{
    async fn run(
        &self,
        input: Authorized<ListOrganizationPipelines>,
    ) -> DomainResult<Fetched<ListOrganizationPipelines>> {
        let query = input.command();
        let page = self
            .pipeline_repo
            .list_by_organization(&query.organization_id, query.pagination.as_ref())
            .await?;
        Ok(input.fetched(page))
    }
}
