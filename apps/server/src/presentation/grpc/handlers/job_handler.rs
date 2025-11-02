use crate::application::dto::{
    CreateJobRequestDto, DeleteJobRequestDto, GetJobRequestDto, ListJobsByPipelineRequestDto,
    ListJobsByStatusRequestDto, ListJobsRequestDto, UpdateJobRequestDto,
};
use crate::domain::value_objects::JobStatus;
use crate::domain::value_objects::{JobId, PipelineId};
use crate::presentation::grpc::mappers::{
    domain_to_proto_metadata, map_domain_error_to_status, proto_to_domain_pagination,
};
// use crate::presentation::grpc::middleware::check_permissions;
use crate::shared::di::AppContainer;
use protocol::services::job::{
    CreateJobRequest, DeleteJobRequest, DeleteJobResponse, GetJobRequest, JobResponse,
    ListJobsByPipelineRequest, ListJobsByPipelineResponse, ListJobsByStatusRequest,
    ListJobsByStatusResponse, ListJobsRequest, ListJobsResponse, UpdateJobRequest,
    job_service_server,
};
use protocol::tonic::{Request, Response, Status};
use std::sync::Arc;

pub struct JobHandler {
    container: Arc<AppContainer>,
}

impl JobHandler {
    pub fn new(container: Arc<AppContainer>) -> Self {
        Self { container }
    }
}

#[async_trait::async_trait]
impl job_service_server::JobService for JobHandler {
    async fn create_job(
        &self,
        request: Request<CreateJobRequest>,
    ) -> Result<Response<JobResponse>, Status> {
        // let pipeline_id = &request.get_ref().pipeline_id;

        // Check RBAC permissions for creating jobs
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     pipeline_id,
        //     "jobs",
        //     "create",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = CreateJobRequestDto {
            pipeline_id: PipelineId::new(req.pipeline_id),
        };

        let response = self
            .container
            .create_job_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(response.into()))
    }

    async fn get_job(
        &self,
        request: Request<GetJobRequest>,
    ) -> Result<Response<JobResponse>, Status> {
        // let job_id = &request.get_ref().job_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     job_id,
        //     "jobs",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = GetJobRequestDto {
            job_id: JobId::new(req.job_id),
        };

        let response = self
            .container
            .get_job_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(response.into()))
    }

    async fn update_job(
        &self,
        request: Request<UpdateJobRequest>,
    ) -> Result<Response<JobResponse>, Status> {
        // let job_id = &request.get_ref().job_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     job_id,
        //     "jobs",
        //     "update",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = UpdateJobRequestDto {
            job_id: JobId::new(req.job_id),
            status: req
                .status
                .map(|s| JobStatus::new(s).map_err(map_domain_error_to_status))
                .transpose()?,
        };

        let response = self
            .container
            .update_job_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(response.into()))
    }

    async fn delete_job(
        &self,
        request: Request<DeleteJobRequest>,
    ) -> Result<Response<DeleteJobResponse>, Status> {
        // let job_id = &request.get_ref().job_id;

        // Check RBAC permissions
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     job_id,
        //     "jobs",
        //     "delete",
        // )
        // .await?;

        let req = request.into_inner();
        let dto = DeleteJobRequestDto {
            job_id: JobId::new(req.job_id),
        };

        let _response = self
            .container
            .delete_job_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        Ok(Response::new(DeleteJobResponse::default()))
    }

    async fn list_jobs(
        &self,
        request: Request<ListJobsRequest>,
    ) -> Result<Response<ListJobsResponse>, Status> {
        // Check RBAC permissions for listing all jobs
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     "*",
        //     "jobs",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let response = self
            .container
            .list_jobs_use_case()
            .execute(ListJobsRequestDto { pagination })
            .await
            .map_err(map_domain_error_to_status)?;

        let jobs = response.jobs.into_iter().map(Into::into).collect();
        let pagination = response.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListJobsResponse { jobs, pagination }))
    }

    async fn list_jobs_by_status(
        &self,
        request: Request<ListJobsByStatusRequest>,
    ) -> Result<Response<ListJobsByStatusResponse>, Status> {
        // Check RBAC permissions for listing jobs by status
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     "*",
        //     "jobs",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let status = JobStatus::new(req.status).map_err(map_domain_error_to_status)?;
        let pagination = proto_to_domain_pagination(req.pagination);

        let dto = ListJobsByStatusRequestDto { status, pagination };

        let responses = self
            .container
            .list_jobs_by_status_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        let jobs = responses.jobs.into_iter().map(Into::into).collect();
        let pagination = responses.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListJobsByStatusResponse { jobs, pagination }))
    }

    async fn list_jobs_by_pipeline(
        &self,
        request: Request<ListJobsByPipelineRequest>,
    ) -> Result<Response<ListJobsByPipelineResponse>, Status> {
        // let pipeline_id = &request.get_ref().pipeline_id;

        // Check RBAC permissions for listing jobs by pipeline
        // check_permissions(
        //     &request,
        //     self.container.rbac_enforcer(),
        //     pipeline_id,
        //     "jobs",
        //     "read",
        // )
        // .await?;

        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let dto = ListJobsByPipelineRequestDto {
            pipeline_id: PipelineId::new(req.pipeline_id),
            pagination,
        };

        let responses = self
            .container
            .list_jobs_by_pipeline_use_case()
            .execute(dto)
            .await
            .map_err(map_domain_error_to_status)?;

        let jobs = responses.jobs.into_iter().map(Into::into).collect();
        let pagination = responses.pagination.map(domain_to_proto_metadata);

        Ok(Response::new(ListJobsByPipelineResponse {
            jobs,
            pagination,
        }))
    }
}
