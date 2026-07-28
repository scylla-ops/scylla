use crate::extract_auth_context;
use crate::grpc::convert::{required, wrap};
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, pipeline_to_proto, pipeline_to_proto_summary,
    proto_to_domain_pagination,
};
use crate::application::{
    AgentDispatch, DispatchOutcome, DispatchUseCases, JobRepository, PermissionService,
    PipelineRepository, PipelineUseCases, ProjectRepository,
};
use scylla_core::domain::entities::{OrganizationId, PipelineId, PipelineNode, ProjectId};
use scylla_core::domain::value_objects::pipeline::{
    EnvKey, EnvVar, NodeId, PipelineName, Shell, Step, WorkingDir,
};
use scylla_core::domain::value_objects::secret::SecretName;
use scylla_protocol::exec::v1 as exec;
use scylla_protocol::pipeline::v1::{
    CreatePipelineRequest, CreatePipelineResponse, DeletePipelineRequest, DeletePipelineResponse,
    EnvVar as ProtoEnvVar, GetPipelineRequest, GetPipelineResponse,
    ListOrganizationPipelinesRequest, ListOrganizationPipelinesResponse, ListPipelinesRequest,
    ListPipelinesResponse, ListProjectPipelinesRequest, ListProjectPipelinesResponse,
    PipelineNode as ProtoPipelineNode, PipelineSummary, RunPipelineRequest, RunPipelineResponse,
    UpdatePipelineRequest, UpdatePipelineResponse, env_var, pipeline_node,
    pipeline_service_server::PipelineService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct PipelineHandler<
    P: PipelineRepository,
    PR: ProjectRepository,
    J: JobRepository,
    PS: PermissionService,
    WD: AgentDispatch,
> {
    use_cases: Arc<PipelineUseCases<P, PR, J, PS>>,
    dispatch_uc: Arc<DispatchUseCases<WD, PS>>,
}

impl<
    P: PipelineRepository,
    PR: ProjectRepository,
    J: JobRepository,
    PS: PermissionService,
    WD: AgentDispatch,
> PipelineHandler<P, PR, J, PS, WD>
{
    pub fn new(
        use_cases: Arc<PipelineUseCases<P, PR, J, PS>>,
        dispatch_uc: Arc<DispatchUseCases<WD, PS>>,
    ) -> Self {
        Self {
            use_cases,
            dispatch_uc,
        }
    }
}

#[async_trait::async_trait]
impl<
    P: PipelineRepository + Send + Sync + 'static,
    PR: ProjectRepository + Send + Sync + 'static,
    J: JobRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
    WD: AgentDispatch + Send + Sync + 'static,
> PipelineService for PipelineHandler<P, PR, J, PS, WD>
{
    async fn create_pipeline(
        &self,
        request: Request<CreatePipelineRequest>,
    ) -> Result<Response<CreatePipelineResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        let name = PipelineName::new(&req.name).map_err(domain_error_to_status)?;
        let project_id = ProjectId::new(&required(req.project_id, "project_id")?);

        let nodes: Vec<PipelineNode> = req
            .nodes
            .into_iter()
            .map(proto_node_to_domain)
            .collect::<Result<_, _>>()?;

        let pipeline = self
            .use_cases
            .create(&caller, name, project_id, nodes)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(CreatePipelineResponse {
            pipeline: Some(pipeline_to_proto(&pipeline)),
        }))
    }

    async fn get_pipeline(
        &self,
        request: Request<GetPipelineRequest>,
    ) -> Result<Response<GetPipelineResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = PipelineId::new(&required(req.pipeline_id, "pipeline_id")?);

        let pipeline = self
            .use_cases
            .get(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(GetPipelineResponse {
            pipeline: Some(pipeline_to_proto(&pipeline)),
        }))
    }

    async fn update_pipeline(
        &self,
        request: Request<UpdatePipelineRequest>,
    ) -> Result<Response<UpdatePipelineResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = PipelineId::new(&required(req.pipeline_id, "pipeline_id")?);

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
                .map(proto_node_to_domain)
                .collect::<Result<_, _>>()?;
            Some(parsed)
        };

        let pipeline = self
            .use_cases
            .update(&caller, &id, name, nodes)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(UpdatePipelineResponse {
            pipeline: Some(pipeline_to_proto(&pipeline)),
        }))
    }

    async fn delete_pipeline(
        &self,
        request: Request<DeletePipelineRequest>,
    ) -> Result<Response<DeletePipelineResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = PipelineId::new(&required(req.pipeline_id, "pipeline_id")?);

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
    ) -> Result<Response<ListProjectPipelinesResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let project_id = ProjectId::new(&required(req.project_id, "project_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_project(&caller, &project_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (pipelines, metadata) = result.into_parts();
        let pipelines: Vec<PipelineSummary> =
            pipelines.iter().map(pipeline_to_proto_summary).collect();

        Ok(Response::new(ListProjectPipelinesResponse {
            pipelines,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_organization_pipelines(
        &self,
        request: Request<ListOrganizationPipelinesRequest>,
    ) -> Result<Response<ListOrganizationPipelinesResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let organization_id =
            OrganizationId::new(&required(req.organization_id, "organization_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_organization(&caller, &organization_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (pipelines, metadata) = result.into_parts();
        let pipelines: Vec<PipelineSummary> =
            pipelines.iter().map(pipeline_to_proto_summary).collect();

        Ok(Response::new(ListOrganizationPipelinesResponse {
            pipelines,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn run_pipeline(
        &self,
        request: Request<RunPipelineRequest>,
    ) -> Result<Response<RunPipelineResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let pipeline_id = PipelineId::new(&required(req.pipeline_id, "pipeline_id")?);

        // Single permission check + load + job mint, all inside the use case.
        let (job, dispatch) = self
            .use_cases
            .run(&caller, &pipeline_id)
            .await
            .map_err(domain_error_to_status)?;

        // Hand the job to a connected, authorized agent. Best-effort: the job
        // is already persisted, so a missing agent leaves it pending rather
        // than failing the request (the use case logs the no-agent case).
        match self.dispatch_uc.dispatch_job(&pipeline_id, &dispatch).await {
            Ok(DispatchOutcome::Dispatched(app_id)) => {
                tracing::info!(job_id = %job.id(), %app_id, "job dispatched to agent");
                // Record attribution for agent stats. Best-effort: the job ran
                // regardless, so a failed write must not fail the request.
                if let Err(e) = self.use_cases.assign_agent(job.id(), &app_id).await {
                    tracing::warn!(job_id = %job.id(), %app_id, error = %e, "failed to record job agent attribution");
                }
            }
            Ok(DispatchOutcome::NoAgentAvailable) => {}
            Err(e) => {
                tracing::warn!(job_id = %job.id(), error = %e, "agent dispatch failed; job left pending");
            }
        }

        // Only the id: callers fetch the run with JobService.GetJob.
        Ok(Response::new(RunPipelineResponse {
            job_id: wrap(job.id().to_string()),
        }))
    }
}

/// Convert a wire `PipelineNode` into the validated domain node. Returns a
/// `Status` (not a `DomainError`) so callers can `?` it directly inside a gRPC
/// handler.
fn proto_node_to_domain(n: ProtoPipelineNode) -> Result<PipelineNode, Status> {
    let node_id = NodeId::new(&required(n.node_id, "node_id")?).map_err(domain_error_to_status)?;
    let deps: Vec<NodeId> = n
        .deps
        .into_iter()
        .map(|d| NodeId::new(&d.value).map_err(domain_error_to_status))
        .collect::<Result<_, _>>()?;
    let working_dir = match n.working_dir.trim() {
        "" => None,
        s => Some(WorkingDir::new(s).map_err(domain_error_to_status)?),
    };
    let env = n
        .env
        .into_iter()
        .map(proto_env_to_domain)
        .collect::<Result<_, _>>()?;
    let step = match n.step {
        Some(pipeline_node::Step::Exec(e)) => {
            Step::exec(e.command, e.args).map_err(domain_error_to_status)?
        }
        Some(pipeline_node::Step::Script(s)) => {
            Step::script(s.script, proto_shell(s.shell)).map_err(domain_error_to_status)?
        }
        None => {
            return Err(Status::invalid_argument(
                "pipeline node is missing its step (exec or script)",
            ));
        }
    };
    Ok(PipelineNode::new(node_id, deps, step, working_dir, env))
}

/// Map a wire env var to the domain: either an inline literal or a reference to
/// a project secret (resolved + decrypted at dispatch time).
fn proto_env_to_domain(e: ProtoEnvVar) -> Result<EnvVar, Status> {
    let key = EnvKey::new(&e.key).map_err(domain_error_to_status)?;
    match e.source {
        Some(env_var::Source::Value(v)) => Ok(EnvVar::literal(key, v)),
        Some(env_var::Source::SecretRef(name)) => {
            let secret = SecretName::new(&name).map_err(domain_error_to_status)?;
            Ok(EnvVar::secret(key, secret))
        }
        None => Err(Status::invalid_argument(format!(
            "env var `{}` has no value",
            e.key
        ))),
    }
}

fn proto_shell(raw: i32) -> Shell {
    match exec::Shell::try_from(raw).unwrap_or_default() {
        exec::Shell::Bash => Shell::Bash,
        exec::Shell::Sh | exec::Shell::Unspecified => Shell::Sh,
    }
}
