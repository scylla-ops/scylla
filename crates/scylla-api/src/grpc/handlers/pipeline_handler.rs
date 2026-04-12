use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, job_to_proto, pipeline_to_proto,
    pipeline_to_proto_summary, proto_to_domain_pagination,
};
use hermes_broker_client::Publisher;
use protocol::services::job::JobResponse;
use protocol::services::pipeline::{
    CreatePipelineRequest, DeletePipelineRequest, DeletePipelineResponse, GetPipelineRequest,
    ListOrganizationPipelinesRequest, ListPipelinesRequest, ListPipelinesResponse,
    ListProjectPipelinesRequest, PipelineResponse, PipelineSummary, RunPipelineRequest,
    UpdatePipelineRequest, pipeline_service_server::PipelineService,
};
use scylla_core::application::ports::{
    JobRepository, PermissionService, PipelineRepository, ProjectRepository,
};
use scylla_core::application::{JobUseCases, PipelineUseCases};
use scylla_core::domain::entities::{Job, OrganizationId, PipelineId, PipelineNode, ProjectId};
use scylla_core::domain::value_objects::permission::policy;
use scylla_core::domain::value_objects::pipeline::JobDispatch;
use scylla_core::domain::value_objects::pipeline::{NodeId, PipelineName};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct PipelineHandler<
    P: PipelineRepository,
    PR: ProjectRepository,
    J: JobRepository,
    PS: PermissionService,
> {
    use_cases: Arc<PipelineUseCases<P, PR>>,
    job_uc: Arc<JobUseCases<J>>,
    permission_checker: Arc<PS>,
    broker_publisher: Arc<Publisher>,
}

impl<P: PipelineRepository, PR: ProjectRepository, J: JobRepository, PS: PermissionService>
    PipelineHandler<P, PR, J, PS>
{
    pub fn new(
        use_cases: Arc<PipelineUseCases<P, PR>>,
        job_uc: Arc<JobUseCases<J>>,
        permission_checker: Arc<PS>,
        broker_publisher: Arc<Publisher>,
    ) -> Self {
        Self {
            use_cases,
            job_uc,
            permission_checker,
            broker_publisher,
        }
    }
}

#[async_trait::async_trait]
impl<
    P: PipelineRepository + Send + Sync + 'static,
    PR: ProjectRepository + Send + Sync + 'static,
    J: JobRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> PipelineService for PipelineHandler<P, PR, J, PS>
{
    async fn create_pipeline(
        &self,
        request: Request<CreatePipelineRequest>,
    ) -> Result<Response<PipelineResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(self, request, policy::pipeline::create(target_project_id));
        let req = request.into_inner();

        let name = PipelineName::new(&req.name).map_err(domain_error_to_status)?;
        let project_id = ProjectId::new(&req.project_id);

        let nodes: Vec<PipelineNode> = req
            .nodes
            .into_iter()
            .map(|n| {
                let node_id = NodeId::new(&n.node_id).map_err(domain_error_to_status)?;
                let deps: Vec<NodeId> = n
                    .deps
                    .iter()
                    .map(|d| NodeId::new(d).map_err(domain_error_to_status))
                    .collect::<Result<_, _>>()?;
                PipelineNode::new(node_id, deps, n.command, n.args).map_err(domain_error_to_status)
            })
            .collect::<Result<_, _>>()?;

        let pipeline = self
            .use_cases
            .create(name, project_id, nodes)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(pipeline_to_proto(&pipeline)))
    }

    async fn get_pipeline(
        &self,
        request: Request<GetPipelineRequest>,
    ) -> Result<Response<PipelineResponse>, Status> {
        let target_pipeline_id = PipelineId::new(&request.get_ref().pipeline_id);
        require_permission!(self, request, policy::pipeline::get(target_pipeline_id));

        let req = request.into_inner();
        let id = PipelineId::new(&req.pipeline_id);

        let pipeline = self
            .use_cases
            .get(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(pipeline_to_proto(&pipeline)))
    }

    async fn update_pipeline(
        &self,
        request: Request<UpdatePipelineRequest>,
    ) -> Result<Response<PipelineResponse>, Status> {
        let target_pipeline_id = PipelineId::new(&request.get_ref().pipeline_id);
        require_permission!(self, request, policy::pipeline::update(target_pipeline_id));

        let req = request.into_inner();
        let id = PipelineId::new(&req.pipeline_id);

        let name = req
            .name
            .map(|n| PipelineName::new(&n))
            .transpose()
            .map_err(domain_error_to_status)?;

        let nodes = if req.nodes.is_empty() {
            None
        } else {
            let parsed: Vec<PipelineNode> = req
                .nodes
                .into_iter()
                .map(|n| {
                    let node_id = NodeId::new(&n.node_id).map_err(domain_error_to_status)?;
                    let deps: Vec<NodeId> = n
                        .deps
                        .iter()
                        .map(|d| NodeId::new(d).map_err(domain_error_to_status))
                        .collect::<Result<_, _>>()?;
                    PipelineNode::new(node_id, deps, n.command, n.args)
                        .map_err(domain_error_to_status)
                })
                .collect::<Result<_, _>>()?;
            Some(parsed)
        };

        let pipeline = self
            .use_cases
            .update(&id, name, nodes)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(pipeline_to_proto(&pipeline)))
    }

    async fn delete_pipeline(
        &self,
        request: Request<DeletePipelineRequest>,
    ) -> Result<Response<DeletePipelineResponse>, Status> {
        let target_pipeline_id = PipelineId::new(&request.get_ref().pipeline_id);
        require_permission!(self, request, policy::pipeline::delete(target_pipeline_id));

        let req = request.into_inner();
        let id = PipelineId::new(&req.pipeline_id);

        self.use_cases
            .delete(&id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeletePipelineResponse {}))
    }

    async fn list_pipelines(
        &self,
        request: Request<ListPipelinesRequest>,
    ) -> Result<Response<ListPipelinesResponse>, Status> {
        require_permission!(self, request, policy::pipeline::list());

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (pipelines, metadata) = result.into_parts();
        let pipelines: Vec<PipelineSummary> =
            pipelines.iter().map(pipeline_to_proto_summary).collect();

        Ok(Response::new(ListPipelinesResponse {
            pipelines,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_project_pipelines(
        &self,
        request: Request<ListProjectPipelinesRequest>,
    ) -> Result<Response<ListPipelinesResponse>, Status> {
        let target_project_id = ProjectId::new(&request.get_ref().project_id);
        require_permission!(
            self,
            request,
            policy::pipeline::list_by_project(target_project_id)
        );

        let req = request.into_inner();
        let project_id = ProjectId::new(&req.project_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_project(&project_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (pipelines, metadata) = result.into_parts();
        let pipelines: Vec<PipelineSummary> =
            pipelines.iter().map(pipeline_to_proto_summary).collect();

        Ok(Response::new(ListPipelinesResponse {
            pipelines,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_organization_pipelines(
        &self,
        request: Request<ListOrganizationPipelinesRequest>,
    ) -> Result<Response<ListPipelinesResponse>, Status> {
        let target_org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::pipeline::list_by_organization(target_org_id)
        );

        let req = request.into_inner();
        let organization_id = OrganizationId::new(&req.organization_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_organization(&organization_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (pipelines, metadata) = result.into_parts();
        let pipelines: Vec<PipelineSummary> =
            pipelines.iter().map(pipeline_to_proto_summary).collect();

        Ok(Response::new(ListPipelinesResponse {
            pipelines,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn run_pipeline(
        &self,
        request: Request<RunPipelineRequest>,
    ) -> Result<Response<JobResponse>, Status> {
        let target_pipeline_id = PipelineId::new(&request.get_ref().pipeline_id);
        require_permission!(self, request, policy::pipeline::run(target_pipeline_id));

        let req = request.into_inner();
        let pipeline_id = PipelineId::new(&req.pipeline_id);

        // 1. Load pipeline
        let pipeline = self
            .use_cases
            .get(&pipeline_id)
            .await
            .map_err(domain_error_to_status)?;

        // 2. Create job from pipeline
        let job = Job::create_from_pipeline(&pipeline);

        // 3. Persist job
        let job = self
            .job_uc
            .create(&job)
            .await
            .map_err(domain_error_to_status)?;

        // 4. Build dispatch message
        let dispatch = JobDispatch {
            job_id: job.id().to_string(),
            pipeline_id: pipeline.id().to_string(),
            nodes: pipeline.nodes().to_vec(),
        };
        let payload = serde_json::to_vec(&dispatch).expect("serialization cannot fail");

        // 5. Publish to broker with reply_to
        self.broker_publisher
            .publish_with_reply(
                "scylla.jobs.dispatch",
                payload,
                format!("scylla.jobs.status.{}", job.id()),
            )
            .await
            .map_err(|_| Status::unavailable("broker unavailable"))?;

        // 6. Return job response
        Ok(Response::new(job_to_proto(&job)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_interceptor::AuthContext;
    use async_trait::async_trait;
    use protocol::services::pipeline::pipeline_service_server::PipelineService;
    use scylla_core::application::ports::services::permission_service::PermissionService;
    use scylla_core::application::ports::{JobRepository, PipelineRepository, ProjectRepository};
    use scylla_core::application::{JobUseCases, PipelineUseCases};
    use scylla_core::domain::entities::UserId;
    use scylla_core::domain::entities::{EntityId, Job, JobId, Pipeline, Project};
    use scylla_core::domain::errors::DomainResult;
    use scylla_core::domain::value_objects::permission::policy::{GroupingPolicy, Policy};
    use scylla_core::domain::value_objects::pipeline::PipelineName;
    use scylla_core::domain::value_objects::project::{ProjectDescription, ProjectName};
    use scylla_core::domain::value_objects::{PaginatedResult, PaginationParams};
    use std::sync::Arc;

    // ── Stubs ──────────────────────────────────────────────────

    #[derive(Default)]
    struct StubPipelineRepo {
        create_fn: Option<Box<dyn Fn(&Pipeline) -> DomainResult<Pipeline> + Send + Sync>>,
        find_by_id_fn: Option<Box<dyn Fn(&PipelineId) -> DomainResult<Pipeline> + Send + Sync>>,
        update_fn: Option<Box<dyn Fn(&Pipeline) -> DomainResult<Pipeline> + Send + Sync>>,
        delete_fn: Option<Box<dyn Fn(&PipelineId) -> DomainResult<()> + Send + Sync>>,
        list_all_fn: Option<Box<dyn Fn() -> DomainResult<PaginatedResult<Pipeline>> + Send + Sync>>,
        list_by_project_fn: Option<
            Box<dyn Fn(&ProjectId) -> DomainResult<PaginatedResult<Pipeline>> + Send + Sync>,
        >,
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
        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Pipeline>> {
            (self.list_all_fn.as_ref().unwrap())()
        }
        async fn list_by_project(
            &self,
            pid: &ProjectId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Pipeline>> {
            (self.list_by_project_fn.as_ref().unwrap())(pid)
        }
        async fn list_by_organization(
            &self,
            _oid: &OrganizationId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Pipeline>> {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct StubProjectRepo {
        find_by_id_fn: Option<Box<dyn Fn(&ProjectId) -> DomainResult<Project> + Send + Sync>>,
    }

    #[async_trait]
    impl ProjectRepository for StubProjectRepo {
        async fn create(&self, _p: &Project) -> DomainResult<Project> {
            unimplemented!()
        }
        async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn update(&self, _p: &Project) -> DomainResult<Project> {
            unimplemented!()
        }
        async fn delete(&self, _id: &ProjectId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Project>> {
            unimplemented!()
        }
        async fn list_active(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Project>> {
            unimplemented!()
        }
        async fn list_by_organization(
            &self,
            _org_id: &OrganizationId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Project>> {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct StubJobRepo;

    #[async_trait]
    impl JobRepository for StubJobRepo {
        async fn create(&self, j: &Job) -> DomainResult<Job> {
            Ok(j.clone())
        }
        async fn find_by_id(&self, _id: &JobId) -> DomainResult<Job> {
            unimplemented!()
        }
        async fn update(&self, j: &Job) -> DomainResult<Job> {
            Ok(j.clone())
        }
        async fn delete(&self, _id: &JobId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            unimplemented!()
        }
        async fn list_by_pipeline(
            &self,
            _pid: &PipelineId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Job>> {
            unimplemented!()
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

    fn test_pipeline() -> Pipeline {
        Pipeline::create(
            PipelineName::new("testpipe").unwrap(),
            ProjectId::generate(),
            vec![
                PipelineNode::new(
                    NodeId::new("step1").unwrap(),
                    vec![],
                    "echo".into(),
                    vec!["hello".into()],
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    fn authed_request<T>(body: T) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut()
            .insert(AuthContext::new(UserId::generate()));
        req
    }

    fn make_handler(
        pipeline_repo: StubPipelineRepo,
        project_repo: StubProjectRepo,
    ) -> PipelineHandler<StubPipelineRepo, StubProjectRepo, StubJobRepo, AllowAll> {
        let uc = Arc::new(PipelineUseCases::new(
            Arc::new(pipeline_repo),
            Arc::new(project_repo),
        ));
        let job_uc = Arc::new(JobUseCases::new(Arc::new(StubJobRepo)));
        let broker_publisher = Arc::new(Publisher::noop());
        PipelineHandler::new(uc, job_uc, Arc::new(AllowAll), broker_publisher)
    }

    // ── Tests ───────────────────────────────────────────────────

    #[tokio::test]
    async fn create_pipeline_returns_proto() {
        let project = Project::create(
            ProjectName::new("proj").unwrap(),
            Some(ProjectDescription::new("d").unwrap()),
            OrganizationId::generate(),
        )
        .unwrap();
        let proj_id = project.id().clone();

        let mut proj_repo = StubProjectRepo::default();
        proj_repo.find_by_id_fn = Some(Box::new(move |_| Ok(project.clone())));

        let mut pipe_repo = StubPipelineRepo::default();
        pipe_repo.create_fn = Some(Box::new(|p| Ok(p.clone())));

        let handler = make_handler(pipe_repo, proj_repo);
        let req = authed_request(CreatePipelineRequest {
            name: "newpipe".into(),
            project_id: proj_id.to_string(),
            nodes: vec![protocol::services::pipeline::PipelineNode {
                node_id: "step1".into(),
                deps: vec![],
                command: "echo".into(),
                args: vec!["hi".into()],
            }],
        });

        let resp = handler.create_pipeline(req).await.unwrap();
        assert_eq!(resp.into_inner().name, "newpipe");
    }

    #[tokio::test]
    async fn get_pipeline_returns_proto() {
        let pipe = test_pipeline();
        let pipe_id_str = pipe.id().to_string();
        let p = pipe.clone();

        let mut repo = StubPipelineRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(p.clone())));

        let handler = make_handler(repo, StubProjectRepo::default());
        let req = authed_request(GetPipelineRequest {
            pipeline_id: pipe_id_str,
        });

        let resp = handler.get_pipeline(req).await.unwrap();
        assert_eq!(resp.into_inner().name, "testpipe");
    }

    #[tokio::test]
    async fn delete_pipeline_success() {
        let pipe = test_pipeline();
        let pipe_id_str = pipe.id().to_string();

        let mut repo = StubPipelineRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(pipe.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let handler = make_handler(repo, StubProjectRepo::default());
        let req = authed_request(DeletePipelineRequest {
            pipeline_id: pipe_id_str,
        });
        assert!(handler.delete_pipeline(req).await.is_ok());
    }

    #[tokio::test]
    async fn list_pipelines_returns_empty() {
        let mut repo = StubPipelineRepo::default();
        repo.list_all_fn = Some(Box::new(|| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let handler = make_handler(repo, StubProjectRepo::default());
        let req = authed_request(ListPipelinesRequest { pagination: None });

        let resp = handler.list_pipelines(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.pipelines.is_empty());
        assert!(inner.pagination.is_some());
    }

    #[tokio::test]
    async fn list_project_pipelines_returns_empty() {
        let project_id = ProjectId::generate();

        let mut repo = StubPipelineRepo::default();
        repo.list_by_project_fn = Some(Box::new(|_| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let handler = make_handler(repo, StubProjectRepo::default());
        let req = authed_request(ListProjectPipelinesRequest {
            project_id: project_id.to_string(),
            pagination: None,
        });

        let resp = handler.list_project_pipelines(req).await.unwrap();
        assert!(resp.into_inner().pipelines.is_empty());
    }
}
