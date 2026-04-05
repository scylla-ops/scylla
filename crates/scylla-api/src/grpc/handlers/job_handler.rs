use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, job_to_proto, proto_to_domain_pagination,
};
use derive_more::Constructor;
use protocol::services::job::{
    DeleteJobRequest, DeleteJobResponse, GetJobRequest, JobResponse, ListJobsRequest,
    ListJobsResponse, ListOrganizationJobsRequest, ListPipelineJobsRequest, ListProjectJobsRequest,
    job_service_server::JobService,
};
use scylla_core::application::JobUseCases;
use scylla_core::application::ports::{JobRepository, PermissionService};
use scylla_core::domain::entities::{JobId, OrganizationId, PipelineId, ProjectId};
use scylla_core::domain::value_objects::permission::policy;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct JobHandler<J: JobRepository, PS: PermissionService> {
    use_cases: Arc<JobUseCases<J>>,
    permission_checker: Arc<PS>,
}

#[async_trait::async_trait]
impl<J: JobRepository + Send + Sync + 'static, PS: PermissionService + Send + Sync + 'static>
    JobService for JobHandler<J, PS>
{
    async fn get_job(
        &self,
        request: Request<GetJobRequest>,
    ) -> Result<Response<JobResponse>, Status> {
        let target_job_id = JobId::new(&request.get_ref().job_id);
        require_permission!(self, request, policy::job::get(target_job_id));

        let req = request.into_inner();
        let id = JobId::new(&req.job_id);

        let job = self
            .use_cases
            .get(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(job_to_proto(&job)))
    }

    async fn delete_job(
        &self,
        request: Request<DeleteJobRequest>,
    ) -> Result<Response<DeleteJobResponse>, Status> {
        let target_job_id = JobId::new(&request.get_ref().job_id);
        require_permission!(self, request, policy::job::delete(target_job_id));

        let req = request.into_inner();
        let id = JobId::new(&req.job_id);

        self.use_cases
            .delete(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteJobResponse {}))
    }

    async fn list_jobs(
        &self,
        request: Request<ListJobsRequest>,
    ) -> Result<Response<ListJobsResponse>, Status> {
        require_permission!(self, request, policy::job::list());

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (jobs, metadata) = result.into_parts();
        let jobs: Vec<JobResponse> = jobs.iter().map(job_to_proto).collect();

        Ok(Response::new(ListJobsResponse {
            jobs,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_pipeline_jobs(
        &self,
        request: Request<ListPipelineJobsRequest>,
    ) -> Result<Response<ListJobsResponse>, Status> {
        require_permission!(self, request, policy::job::list_by_pipeline());

        let req = request.into_inner();
        let pipeline_id = PipelineId::new(&req.pipeline_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_pipeline(&pipeline_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (jobs, metadata) = result.into_parts();
        let jobs: Vec<JobResponse> = jobs.iter().map(job_to_proto).collect();

        Ok(Response::new(ListJobsResponse {
            jobs,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_project_jobs(
        &self,
        request: Request<ListProjectJobsRequest>,
    ) -> Result<Response<ListJobsResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(
            self,
            request,
            policy::job::list_by_project(target_project_id)
        );

        let req = request.into_inner();
        let project_id = ProjectId::new(&req.project_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_project(&project_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (jobs, metadata) = result.into_parts();
        let jobs: Vec<JobResponse> = jobs.iter().map(job_to_proto).collect();

        Ok(Response::new(ListJobsResponse {
            jobs,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_organization_jobs(
        &self,
        request: Request<ListOrganizationJobsRequest>,
    ) -> Result<Response<ListJobsResponse>, Status> {
        let target_org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::job::list_by_organization(target_org_id)
        );

        let req = request.into_inner();
        let organization_id = OrganizationId::new(&req.organization_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_organization(&organization_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (jobs, metadata) = result.into_parts();
        let jobs: Vec<JobResponse> = jobs.iter().map(job_to_proto).collect();

        Ok(Response::new(ListJobsResponse {
            jobs,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_interceptor::AuthContext;
    use async_trait::async_trait;
    use protocol::services::job::job_service_server::JobService;
    use scylla_core::application::JobUseCases;
    use scylla_core::application::ports::JobRepository;
    use scylla_core::application::ports::services::permission_service::PermissionService;
    use scylla_core::domain::entities::UserId;
    use scylla_core::domain::entities::{EntityId, Job, Pipeline, PipelineNode};
    use scylla_core::domain::errors::{DomainError, DomainResult};
    use scylla_core::domain::value_objects::permission::policy::{GroupingPolicy, Policy};
    use scylla_core::domain::value_objects::pipeline::{NodeId, PipelineName};
    use scylla_core::domain::value_objects::{PaginatedResult, PaginationParams};
    use std::sync::Arc;

    // ── Stubs ──────────────────────────────────────────────────

    #[derive(Default)]
    struct StubJobRepo {
        create_fn: Option<Box<dyn Fn(&Job) -> DomainResult<Job> + Send + Sync>>,
        find_by_id_fn: Option<Box<dyn Fn(&JobId) -> DomainResult<Job> + Send + Sync>>,
        delete_fn: Option<Box<dyn Fn(&JobId) -> DomainResult<()> + Send + Sync>>,
        list_all_fn: Option<Box<dyn Fn() -> DomainResult<PaginatedResult<Job>> + Send + Sync>>,
        list_by_pipeline_fn:
            Option<Box<dyn Fn(&PipelineId) -> DomainResult<PaginatedResult<Job>> + Send + Sync>>,
    }

    #[async_trait]
    impl JobRepository for StubJobRepo {
        async fn create(&self, j: &Job) -> DomainResult<Job> {
            (self.create_fn.as_ref().unwrap())(j)
        }
        async fn find_by_id(&self, id: &JobId) -> DomainResult<Job> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn update(&self, _j: &Job) -> DomainResult<Job> {
            unimplemented!()
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

    struct AllowAll;

    #[async_trait]
    impl PermissionService for AllowAll {
        async fn check(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> {
            Ok(true)
        }
        async fn add_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> {
            Ok(true)
        }
        async fn remove_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> {
            Ok(true)
        }
        async fn list_policies(&self, _s: Option<&str>) -> DomainResult<Vec<(String, Policy)>> {
            Ok(vec![])
        }
        async fn add_grouping_policy(
            &self,
            _s: impl EntityId,
            _p: GroupingPolicy,
        ) -> DomainResult<bool> {
            Ok(true)
        }
        async fn remove_grouping_policy(
            &self,
            _s: impl EntityId,
            _p: GroupingPolicy,
        ) -> DomainResult<bool> {
            Ok(true)
        }
        async fn list_grouping_policies(
            &self,
            _s: Option<&str>,
        ) -> DomainResult<Vec<(String, GroupingPolicy)>> {
            Ok(vec![])
        }
    }

    // ── Helpers ─────────────────────────────────────────────────

    fn test_job() -> Job {
        let pipeline = Pipeline::create(
            PipelineName::new("pipe").unwrap(),
            ProjectId::generate(),
            vec![
                PipelineNode::new(NodeId::new("step1").unwrap(), vec![], "echo".into(), vec![])
                    .unwrap(),
            ],
        )
        .unwrap();
        Job::create_from_pipeline(&pipeline)
    }

    fn authed_request<T>(body: T) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut()
            .insert(AuthContext::new(UserId::generate()));
        req
    }

    fn make_handler(job_repo: StubJobRepo) -> JobHandler<StubJobRepo, AllowAll> {
        let uc = Arc::new(JobUseCases::new(Arc::new(job_repo)));
        JobHandler::new(uc, Arc::new(AllowAll))
    }

    // ── Tests ───────────────────────────────────────────────────

    #[tokio::test]
    async fn get_job_returns_proto() {
        let job = test_job();
        let job_id_str = job.id().to_string();
        let j = job.clone();

        let mut repo = StubJobRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(j.clone())));

        let handler = make_handler(repo);
        let req = authed_request(GetJobRequest { job_id: job_id_str });

        let resp = handler.get_job(req).await.unwrap();
        assert!(!resp.into_inner().job_id.is_empty());
    }

    #[tokio::test]
    async fn get_job_not_found() {
        let mut repo = StubJobRepo::default();
        repo.find_by_id_fn = Some(Box::new(|id| {
            Err(DomainError::not_found("Job", id.to_string()))
        }));

        let handler = make_handler(repo);
        let req = authed_request(GetJobRequest {
            job_id: "nonexistent".into(),
        });

        let err = handler.get_job(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn delete_job_success() {
        let job = test_job();
        let job_id_str = job.id().to_string();

        let mut repo = StubJobRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(job.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let handler = make_handler(repo);
        let req = authed_request(DeleteJobRequest { job_id: job_id_str });
        assert!(handler.delete_job(req).await.is_ok());
    }

    #[tokio::test]
    async fn list_jobs_returns_empty() {
        let mut repo = StubJobRepo::default();
        repo.list_all_fn = Some(Box::new(|| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let handler = make_handler(repo);
        let req = authed_request(ListJobsRequest { pagination: None });

        let resp = handler.list_jobs(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.jobs.is_empty());
        assert!(inner.pagination.is_some());
    }

    #[tokio::test]
    async fn list_pipeline_jobs_returns_empty() {
        let pipeline_id = PipelineId::generate();

        let mut repo = StubJobRepo::default();
        repo.list_by_pipeline_fn = Some(Box::new(|_| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let handler = make_handler(repo);
        let req = authed_request(ListPipelineJobsRequest {
            pipeline_id: pipeline_id.to_string(),
            pagination: None,
        });

        let resp = handler.list_pipeline_jobs(req).await.unwrap();
        assert!(resp.into_inner().jobs.is_empty());
    }
}
