use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, job_to_proto, pipeline_to_proto,
    pipeline_to_proto_summary, proto_to_domain_pagination,
};
use hermes_broker_client::Publisher;
use scylla_core::application::PipelineUseCases;
use scylla_core::application::{
    JobRepository, PermissionService, PipelineRepository, ProjectRepository,
};
use scylla_core::domain::entities::{OrganizationId, PipelineId, PipelineNode, ProjectId};
use scylla_core::domain::value_objects::pipeline::{NodeId, PipelineName};
use scylla_protocol::services::job::JobResponse;
use scylla_protocol::services::pipeline::{
    CreatePipelineRequest, DeletePipelineRequest, DeletePipelineResponse, GetPipelineRequest,
    ListOrganizationPipelinesRequest, ListPipelinesRequest, ListPipelinesResponse,
    ListProjectPipelinesRequest, PipelineResponse, PipelineSummary, RunPipelineRequest,
    UpdatePipelineRequest, pipeline_service_server::PipelineService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct PipelineHandler<
    P: PipelineRepository,
    PR: ProjectRepository,
    J: JobRepository,
    PS: PermissionService,
> {
    use_cases: Arc<PipelineUseCases<P, PR, J, PS>>,
    broker_publisher: Arc<Publisher>,
}

impl<P: PipelineRepository, PR: ProjectRepository, J: JobRepository, PS: PermissionService>
    PipelineHandler<P, PR, J, PS>
{
    pub fn new(
        use_cases: Arc<PipelineUseCases<P, PR, J, PS>>,
        broker_publisher: Arc<Publisher>,
    ) -> Self {
        Self {
            use_cases,
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
        let caller = caller!(request);
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
            .create(&caller, name, project_id, nodes)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(pipeline_to_proto(&pipeline)))
    }

    async fn get_pipeline(
        &self,
        request: Request<GetPipelineRequest>,
    ) -> Result<Response<PipelineResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = PipelineId::new(&req.pipeline_id);

        let pipeline = self
            .use_cases
            .get(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(pipeline_to_proto(&pipeline)))
    }

    async fn update_pipeline(
        &self,
        request: Request<UpdatePipelineRequest>,
    ) -> Result<Response<PipelineResponse>, Status> {
        let caller = caller!(request);
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
            .update(&caller, &id, name, nodes)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(pipeline_to_proto(&pipeline)))
    }

    async fn delete_pipeline(
        &self,
        request: Request<DeletePipelineRequest>,
    ) -> Result<Response<DeletePipelineResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = PipelineId::new(&req.pipeline_id);

        self.use_cases
            .delete(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeletePipelineResponse {}))
    }

    async fn list_pipelines(
        &self,
        request: Request<ListPipelinesRequest>,
    ) -> Result<Response<ListPipelinesResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(&caller, pagination.as_ref())
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
        let caller = caller!(request);
        let req = request.into_inner();
        let project_id = ProjectId::new(&req.project_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_project(&caller, &project_id, pagination.as_ref())
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
        let caller = caller!(request);
        let req = request.into_inner();
        let organization_id = OrganizationId::new(&req.organization_id);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_organization(&caller, &organization_id, pagination.as_ref())
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
        let caller = caller!(request);
        let req = request.into_inner();
        let pipeline_id = PipelineId::new(&req.pipeline_id);

        // Single permission check + load + job mint, all inside the use case.
        let (job, dispatch) = self
            .use_cases
            .run(&caller, &pipeline_id)
            .await
            .map_err(domain_error_to_status)?;

        let payload = serde_json::to_vec(&dispatch).expect("serialization cannot fail");

        // Dispatch is best-effort IO at the handler boundary; the job is
        // already persisted, so failure here is reported but does not roll
        // back the job row (the recorder reconciles status later).
        self.broker_publisher
            .publish_with_reply(
                "scylla.jobs.dispatch",
                payload,
                format!("scylla.jobs.status.{}", job.id()),
            )
            .await
            .map_err(|_| Status::unavailable("broker unavailable"))?;

        Ok(Response::new(job_to_proto(&job)))
    }
}
