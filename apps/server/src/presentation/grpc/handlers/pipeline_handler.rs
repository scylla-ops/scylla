use crate::application::dto::{
    CreatePipelineRequestDto, DeletePipelineRequestDto, GetPipelineRequestDto,
    ListPipelinesRequestDto, UpdatePipelineRequestDto,
};
use crate::domain::value_objects::PipelineId;
use crate::presentation::grpc::mappers::{domain_to_proto_metadata, proto_to_domain_pagination};
// use crate::presentation::grpc::middleware::check_permissions;
use crate::shared::di::AppContainer;
use derive_more::Constructor;
use protocol::services::pipeline::{
    CreatePipelineRequest, DeletePipelineRequest, DeletePipelineResponse, GetPipelineRequest,
    ListPipelinesRequest, ListPipelinesResponse, PipelineResponse, UpdatePipelineRequest,
    pipeline_server,
};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

#[derive(Constructor)]
pub struct PipelineHandler {
    container: Arc<AppContainer>,
}

#[async_trait::async_trait]
impl pipeline_server::Pipeline for PipelineHandler {
    async fn create_pipeline(
        &self,
        request: Request<CreatePipelineRequest>,
    ) -> Result<Response<PipelineResponse>, Status> {
        // Check RBAC permissions for creating pipelines
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     "*",
        //     "pipelines",
        //     "create",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = match req.content {
            Some(proto_pipeline_content) => {
                let domain_pipeline_content: PipelineContent<TemplateProfile> =
                    proto_pipeline_content.try_into()?;
                CreatePipelineRequestDto {
                    content: domain_pipeline_content,
                }
            }
            None => {
                return Err(Status::invalid_argument("Pipeline content is required"));
            }
        };

        let response = self
            .container
            .create_pipeline_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn get_pipeline(
        &self,
        request: Request<GetPipelineRequest>,
    ) -> Result<Response<PipelineResponse>, Status> {
        // let pipeline_id = &request.get_ref().pipeline_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     pipeline_id,
        //     "pipelines",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = GetPipelineRequestDto {
            pipeline_id: PipelineId::new(req.pipeline_id),
        };

        let response = self.container.get_pipeline_use_case().execute(dto).await?;

        Ok(Response::new(response.into()))
    }

    async fn update_pipeline(
        &self,
        request: Request<UpdatePipelineRequest>,
    ) -> Result<Response<PipelineResponse>, Status> {
        // let pipeline_id_str = &request.get_ref().pipeline_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     pipeline_id_str,
        //     "pipelines",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();
        let pipeline_id = PipelineId::new(req.pipeline_id);
        let dto = UpdatePipelineRequestDto {
            pipeline_id,
            content: match req.content {
                Some(proto_pipeline_content) => proto_pipeline_content.try_into()?,
                None => {
                    return Err(Status::invalid_argument(
                        "Pipeline content is required for update",
                    ));
                }
            },
        };

        let response = self
            .container
            .update_pipeline_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn delete_pipeline(
        &self,
        request: Request<DeletePipelineRequest>,
    ) -> Result<Response<DeletePipelineResponse>, Status> {
        // let pipeline_id = &request.get_ref().pipeline_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     pipeline_id,
        //     "pipelines",
        //     "delete",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = DeletePipelineRequestDto {
            pipeline_id: PipelineId::new(req.pipeline_id),
        };

        let _response = self
            .container
            .delete_pipeline_use_case()
            .execute(dto)
            .await?;

        Ok(Response::new(DeletePipelineResponse::default()))
    }

    async fn list_pipelines(
        &self,
        request: Request<ListPipelinesRequest>,
    ) -> Result<Response<ListPipelinesResponse>, Status> {
        // Check RBAC permissions for listing all pipelines
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     "*",
        //     "pipelines",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let response = self
            .container
            .list_pipelines_use_case()
            .execute(ListPipelinesRequestDto { pagination })
            .await?;

        let pipelines = response.pipelines.into_iter().map(Into::into).collect();
        let pagination = response.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListPipelinesResponse {
            pipelines,
            pagination,
        }))
    }
}
