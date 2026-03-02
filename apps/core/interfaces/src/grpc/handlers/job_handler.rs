use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, job_to_proto, proto_to_domain_pagination,
};
use application::JobUseCases;
use derive_more::Constructor;
use domain::entities::{JobId, OrganizationId, PipelineId, ProjectId};
use domain::ports::{JobRepository, PermissionService};
use domain::value_objects::permission::policy;
use protocol::services::job::{
    DeleteJobRequest, DeleteJobResponse, GetJobRequest, JobResponse, ListJobsRequest,
    ListJobsResponse, ListOrganizationJobsRequest, ListPipelineJobsRequest,
    ListProjectJobsRequest, job_service_server::JobService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct JobHandler<J: JobRepository, PS: PermissionService> {
    use_cases: Arc<JobUseCases<J>>,
    permission_checker: Arc<PS>,
}

#[async_trait::async_trait]
impl<
    J: JobRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> JobService for JobHandler<J, PS>
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
