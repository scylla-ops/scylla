use derive_more::Constructor;
use domain::entities::{OrganizationId, Pipeline, PipelineId, PipelineNode, ProjectId};
use domain::errors::DomainResult;
use domain::ports::{PipelineRepository, ProjectRepository};
use domain::value_objects::pipeline::PipelineName;
use domain::value_objects::{PaginatedResult, PaginationParams};
use std::sync::Arc;

#[derive(Constructor)]
pub struct PipelineUseCases<P: PipelineRepository, PR: ProjectRepository> {
    pipeline_repo: Arc<P>,
    project_repo: Arc<PR>,
}

impl<P: PipelineRepository, PR: ProjectRepository> PipelineUseCases<P, PR> {
    pub async fn create(
        &self,
        name: PipelineName,
        project_id: ProjectId,
        nodes: Vec<PipelineNode>,
    ) -> DomainResult<Pipeline> {
        self.project_repo.find_by_id(&project_id).await?;
        let pipeline = Pipeline::create(name, project_id, nodes)?;
        self.pipeline_repo.create(&pipeline).await
    }

    pub async fn get(&self, id: &PipelineId) -> DomainResult<Pipeline> {
        self.pipeline_repo.find_by_id(id).await
    }

    pub async fn update(
        &self,
        id: &PipelineId,
        name: Option<PipelineName>,
        nodes: Option<Vec<PipelineNode>>,
    ) -> DomainResult<Pipeline> {
        let mut pipeline = self.pipeline_repo.find_by_id(id).await?;

        if let Some(new_name) = name {
            pipeline.update_name(new_name)?;
        }
        if let Some(new_nodes) = nodes {
            pipeline.update_nodes(new_nodes)?;
        }

        self.pipeline_repo.update(&pipeline).await
    }

    pub async fn delete(&self, id: &PipelineId) -> DomainResult<()> {
        self.pipeline_repo.find_by_id(id).await?;
        self.pipeline_repo.delete(id).await
    }

    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.pipeline_repo.list_all(pagination).await
    }

    pub async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.pipeline_repo
            .list_by_project(project_id, pagination)
            .await
    }

    pub async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.pipeline_repo
            .list_by_organization(organization_id, pagination)
            .await
    }
}
