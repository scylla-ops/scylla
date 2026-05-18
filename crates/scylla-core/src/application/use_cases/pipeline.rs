use crate::application::ports::{PipelineRepository, ProjectRepository};
use crate::domain::entities::{OrganizationId, Pipeline, PipelineId, PipelineNode, ProjectId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::pipeline::PipelineName;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct PipelineUseCases<P: PipelineRepository, PR: ProjectRepository> {
    pipeline_repo: Arc<P>,
    project_repo: Arc<PR>,
}

impl<P: PipelineRepository, PR: ProjectRepository> PipelineUseCases<P, PR> {
    #[instrument(skip(self, nodes), fields(name = %name, project_id = %project_id))]
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

    #[instrument(skip(self), fields(pipeline_id = %id))]
    pub async fn get(&self, id: &PipelineId) -> DomainResult<Pipeline> {
        self.pipeline_repo.find_by_id(id).await
    }

    #[instrument(skip(self, nodes), fields(pipeline_id = %id))]
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

    #[instrument(skip(self), fields(pipeline_id = %id))]
    pub async fn delete(&self, id: &PipelineId) -> DomainResult<()> {
        self.pipeline_repo.find_by_id(id).await?;
        self.pipeline_repo.delete(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.pipeline_repo.list_all(pagination).await
    }

    #[instrument(skip(self), fields(project_id = %project_id))]
    pub async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.pipeline_repo
            .list_by_project(project_id, pagination)
            .await
    }

    #[instrument(skip(self), fields(org_id = %organization_id))]
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

