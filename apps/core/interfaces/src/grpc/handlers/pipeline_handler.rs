use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, pipeline_to_proto,
    proto_to_domain_pagination,
};
use application::PipelineUseCases;
use derive_more::Constructor;
use domain::entities::{OrganizationId, PipelineId, PipelineNode, ProjectId};
use domain::ports::{PermissionService, PipelineRepository, ProjectRepository};
use domain::value_objects::permission::policy;
use domain::value_objects::pipeline::{NodeId, PipelineName};
use protocol::services::pipeline::{
    CreatePipelineRequest, DeletePipelineRequest, DeletePipelineResponse, GetPipelineRequest,
    ListOrganizationPipelinesRequest, ListPipelinesRequest, ListPipelinesResponse,
    ListProjectPipelinesRequest, PipelineResponse, UpdatePipelineRequest,
    pipeline_service_server::PipelineService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct PipelineHandler<P: PipelineRepository, PR: ProjectRepository, PS: PermissionService> {
    use_cases: Arc<PipelineUseCases<P, PR>>,
    permission_checker: Arc<PS>,
}

#[async_trait::async_trait]
impl<
    P: PipelineRepository + Send + Sync + 'static,
    PR: ProjectRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> PipelineService for PipelineHandler<P, PR, PS>
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
                PipelineNode::new(node_id, deps, n.command, n.args)
                    .map_err(domain_error_to_status)
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
        let pipelines: Vec<PipelineResponse> =
            pipelines.iter().map(pipeline_to_proto).collect();

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
        let pipelines: Vec<PipelineResponse> =
            pipelines.iter().map(pipeline_to_proto).collect();

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
        let pipelines: Vec<PipelineResponse> =
            pipelines.iter().map(pipeline_to_proto).collect();

        Ok(Response::new(ListPipelinesResponse {
            pipelines,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }
}
