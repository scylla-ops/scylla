use crate::application::ports::JobRepository;
use crate::domain::entities::{Job, JobId, OrganizationId, PipelineId, ProjectId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct JobUseCases<J: JobRepository> {
    job_repo: Arc<J>,
}

impl<J: JobRepository> JobUseCases<J> {
    #[instrument(skip(self, job))]
    pub async fn create(&self, job: &Job) -> DomainResult<Job> {
        self.job_repo.create(job).await
    }

    #[instrument(skip(self), fields(job_id = %id))]
    pub async fn get(&self, id: &JobId) -> DomainResult<Job> {
        self.job_repo.find_by_id(id).await
    }

    #[instrument(skip(self, job))]
    pub async fn update(&self, job: &Job) -> DomainResult<Job> {
        self.job_repo.update(job).await
    }

    #[instrument(skip(self), fields(job_id = %id))]
    pub async fn delete(&self, id: &JobId) -> DomainResult<()> {
        self.job_repo.find_by_id(id).await?;
        self.job_repo.delete(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo.list_all(pagination).await
    }

    #[instrument(skip(self), fields(pipeline_id = %pipeline_id))]
    pub async fn list_by_pipeline(
        &self,
        pipeline_id: &PipelineId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo
            .list_by_pipeline(pipeline_id, pagination)
            .await
    }

    #[instrument(skip(self), fields(project_id = %project_id))]
    pub async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo.list_by_project(project_id, pagination).await
    }

    #[instrument(skip(self), fields(org_id = %organization_id))]
    pub async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        self.job_repo
            .list_by_organization(organization_id, pagination)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::JobRepository;
    use crate::domain::entities::{Pipeline, PipelineNode};
    use crate::domain::errors::DomainError;
    use crate::domain::value_objects::pipeline::{NodeId, PipelineName};
    use async_trait::async_trait;
    use std::sync::Arc;

    #[derive(Default)]
    struct StubJobRepo {
        create_fn: Option<Box<dyn Fn(&Job) -> DomainResult<Job> + Send + Sync>>,
        find_by_id_fn: Option<Box<dyn Fn(&JobId) -> DomainResult<Job> + Send + Sync>>,
        update_fn: Option<Box<dyn Fn(&Job) -> DomainResult<Job> + Send + Sync>>,
        delete_fn: Option<Box<dyn Fn(&JobId) -> DomainResult<()> + Send + Sync>>,
        list_all_fn: Option<Box<dyn Fn() -> DomainResult<PaginatedResult<Job>> + Send + Sync>>,
        list_by_pipeline_fn:
            Option<Box<dyn Fn(&PipelineId) -> DomainResult<PaginatedResult<Job>> + Send + Sync>>,
    }

    #[async_trait]
    impl JobRepository for StubJobRepo {
        async fn create(&self, job: &Job) -> DomainResult<Job> {
            (self.create_fn.as_ref().unwrap())(job)
        }
        async fn find_by_id(&self, id: &JobId) -> DomainResult<Job> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn update(&self, job: &Job) -> DomainResult<Job> {
            (self.update_fn.as_ref().unwrap())(job)
        }
        async fn delete(&self, id: &JobId) -> DomainResult<()> {
            (self.delete_fn.as_ref().unwrap())(id)
        }
        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            (self.list_all_fn.as_ref().unwrap())()
        }
        async fn list_by_pipeline(
            &self,
            pid: &PipelineId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            (self.list_by_pipeline_fn.as_ref().unwrap())(pid)
        }
        async fn list_by_project(
            &self,
            _pid: &ProjectId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            unimplemented!()
        }
        async fn list_by_organization(
            &self,
            _oid: &OrganizationId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            unimplemented!()
        }
    }

    fn test_job() -> Job {
        let node = PipelineNode::new(NodeId::new("step1").unwrap(), vec![], "echo".into(), vec![])
            .unwrap();
        let pipeline = Pipeline::create(
            PipelineName::new("test").unwrap(),
            ProjectId::generate(),
            vec![node],
        )
        .unwrap();
        Job::create_from_pipeline(&pipeline)
    }

    fn make_uc(repo: StubJobRepo) -> JobUseCases<StubJobRepo> {
        JobUseCases::new(Arc::new(repo))
    }

    #[tokio::test]
    async fn create_job() {
        let job = test_job();
        let mut repo = StubJobRepo::default();
        repo.create_fn = Some(Box::new(|j| Ok(j.clone())));

        let uc = make_uc(repo);
        let result = uc.create(&job).await.unwrap();
        assert_eq!(result.node_executions().len(), 1);
    }

    #[tokio::test]
    async fn get_job() {
        let job = test_job();
        let mut repo = StubJobRepo::default();
        let j = job.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(j.clone())));

        let uc = make_uc(repo);
        let result = uc.get(job.id()).await.unwrap();
        assert_eq!(result.id(), job.id());
    }

    #[tokio::test]
    async fn delete_job() {
        let job = test_job();
        let mut repo = StubJobRepo::default();
        let j = job.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(j.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let uc = make_uc(repo);
        assert!(uc.delete(job.id()).await.is_ok());
    }

    #[tokio::test]
    async fn delete_nonexistent_job() {
        let mut repo = StubJobRepo::default();
        repo.find_by_id_fn = Some(Box::new(|id| {
            Err(DomainError::not_found("Job", id.to_string()))
        }));

        let uc = make_uc(repo);
        let result = uc.delete(&JobId::generate()).await;
        assert!(matches!(result.unwrap_err(), DomainError::NotFound { .. }));
    }

    #[tokio::test]
    async fn list_by_pipeline() {
        let mut repo = StubJobRepo::default();
        repo.list_by_pipeline_fn = Some(Box::new(|_| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let uc = make_uc(repo);
        let result = uc
            .list_by_pipeline(&PipelineId::generate(), None)
            .await
            .unwrap();
        assert!(result.items().is_empty());
    }
}
