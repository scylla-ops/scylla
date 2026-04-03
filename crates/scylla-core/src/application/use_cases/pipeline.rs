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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{PipelineRepository, ProjectRepository};
    use crate::domain::entities::Project;
    use crate::domain::errors::DomainError;
    use crate::domain::value_objects::pipeline::NodeId;
    use crate::domain::value_objects::project::ProjectName;
    use async_trait::async_trait;
    use std::sync::Arc;

    #[derive(Default)]
    struct StubPipelineRepo {
        create_fn: Option<Box<dyn Fn(&Pipeline) -> DomainResult<Pipeline> + Send + Sync>>,
        find_by_id_fn: Option<Box<dyn Fn(&PipelineId) -> DomainResult<Pipeline> + Send + Sync>>,
        update_fn: Option<Box<dyn Fn(&Pipeline) -> DomainResult<Pipeline> + Send + Sync>>,
        delete_fn: Option<Box<dyn Fn(&PipelineId) -> DomainResult<()> + Send + Sync>>,
        list_all_fn: Option<Box<dyn Fn() -> DomainResult<PaginatedResult<Pipeline>> + Send + Sync>>,
        list_by_project_fn: Option<Box<dyn Fn(&ProjectId) -> DomainResult<PaginatedResult<Pipeline>> + Send + Sync>>,
    }

    #[async_trait]
    impl PipelineRepository for StubPipelineRepo {
        async fn create(&self, p: &Pipeline) -> DomainResult<Pipeline> {
            (self.create_fn.as_ref().unwrap())(p)
        }
        async fn find_by_id(&self, id: &PipelineId) -> DomainResult<Pipeline> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn update(&self, p: &Pipeline) -> DomainResult<Pipeline> {
            (self.update_fn.as_ref().unwrap())(p)
        }
        async fn delete(&self, id: &PipelineId) -> DomainResult<()> {
            (self.delete_fn.as_ref().unwrap())(id)
        }
        async fn list_all(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Pipeline>> {
            (self.list_all_fn.as_ref().unwrap())()
        }
        async fn list_by_project(&self, pid: &ProjectId, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Pipeline>> {
            (self.list_by_project_fn.as_ref().unwrap())(pid)
        }
        async fn list_by_organization(&self, _oid: &OrganizationId, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Pipeline>> {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct StubProjectRepo {
        find_by_id_fn: Option<Box<dyn Fn(&ProjectId) -> DomainResult<Project> + Send + Sync>>,
    }

    #[async_trait]
    impl ProjectRepository for StubProjectRepo {
        async fn create(&self, _p: &Project) -> DomainResult<Project> { unimplemented!() }
        async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn update(&self, _p: &Project) -> DomainResult<Project> { unimplemented!() }
        async fn delete(&self, _id: &ProjectId) -> DomainResult<()> { unimplemented!() }
        async fn list_all(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Project>> { unimplemented!() }
        async fn list_active(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Project>> { unimplemented!() }
    }

    fn test_project() -> Project {
        Project::create(
            ProjectName::new("Test").unwrap(),
            None,
            OrganizationId::generate(),
        ).unwrap()
    }

    fn test_node() -> PipelineNode {
        PipelineNode::new(
            NodeId::new("step1").unwrap(),
            vec![],
            "echo".into(),
            vec!["hello".into()],
        ).unwrap()
    }

    fn make_uc(
        pipeline_repo: StubPipelineRepo,
        project_repo: StubProjectRepo,
    ) -> PipelineUseCases<StubPipelineRepo, StubProjectRepo> {
        PipelineUseCases::new(Arc::new(pipeline_repo), Arc::new(project_repo))
    }

    #[tokio::test]
    async fn create_success() {
        let project = test_project();
        let mut project_repo = StubProjectRepo::default();
        let p = project.clone();
        project_repo.find_by_id_fn = Some(Box::new(move |_| Ok(p.clone())));

        let mut pipeline_repo = StubPipelineRepo::default();
        pipeline_repo.create_fn = Some(Box::new(|p| Ok(p.clone())));

        let uc = make_uc(pipeline_repo, project_repo);
        let name = PipelineName::new("my-pipeline").unwrap();
        let result = uc.create(name, project.id().clone(), vec![test_node()]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nodes().len(), 1);
    }

    #[tokio::test]
    async fn create_nonexistent_project() {
        let mut project_repo = StubProjectRepo::default();
        project_repo.find_by_id_fn = Some(Box::new(|id| {
            Err(DomainError::not_found("Project", id.to_string()))
        }));

        let uc = make_uc(StubPipelineRepo::default(), project_repo);
        let name = PipelineName::new("my-pipeline").unwrap();
        let result = uc.create(name, ProjectId::generate(), vec![test_node()]).await;
        assert!(matches!(result.unwrap_err(), DomainError::NotFound { .. }));
    }

    #[tokio::test]
    async fn get_pipeline() {
        let project = test_project();
        let pipeline = Pipeline::create(
            PipelineName::new("test").unwrap(),
            project.id().clone(),
            vec![test_node()],
        ).unwrap();

        let mut repo = StubPipelineRepo::default();
        let pl = pipeline.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(pl.clone())));

        let uc = make_uc(repo, StubProjectRepo::default());
        let result = uc.get(pipeline.id()).await.unwrap();
        assert_eq!(result.name().as_str(), "test");
    }

    #[tokio::test]
    async fn delete_pipeline() {
        let project = test_project();
        let pipeline = Pipeline::create(
            PipelineName::new("test").unwrap(),
            project.id().clone(),
            vec![test_node()],
        ).unwrap();

        let mut repo = StubPipelineRepo::default();
        let pl = pipeline.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(pl.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let uc = make_uc(repo, StubProjectRepo::default());
        assert!(uc.delete(pipeline.id()).await.is_ok());
    }

    #[tokio::test]
    async fn list_by_project() {
        let mut repo = StubPipelineRepo::default();
        repo.list_by_project_fn = Some(Box::new(|_| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let uc = make_uc(repo, StubProjectRepo::default());
        let result = uc.list_by_project(&ProjectId::generate(), None).await.unwrap();
        assert!(result.items().is_empty());
    }
}
