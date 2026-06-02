use crate::extract_auth_context;
use crate::grpc::convert::{optional, required};
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, job_log_to_proto, job_to_proto,
    proto_to_domain_pagination,
};
use crate::grpc::streaming::spawn_log_forwarder;
use derive_more::Constructor;
use scylla_core::application::{
    JobLogRepository, JobLogStreamPort, JobRepository, PermissionService,
};
use scylla_core::application::{JobLogStreamUseCase, JobLogUseCases, JobUseCases};
use scylla_core::domain::entities::{JobId, OrganizationId, PipelineId, ProjectId};
use scylla_core::domain::value_objects::PaginationMetadata;
use scylla_core::domain::value_objects::pipeline::NodeId;
use scylla_protocol::services::job::{
    DeleteJobRequest, DeleteJobResponse, GetJobRequest, JobLogEvent, JobResponse,
    ListJobLogsRequest, ListJobLogsResponse, ListJobsRequest, ListJobsResponse,
    ListOrganizationJobsRequest, ListPipelineJobsRequest, ListProjectJobsRequest,
    TailJobLogsRequest, job_service_server::JobService,
};
use std::pin::Pin;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct JobHandler<
    J: JobRepository,
    L: JobLogRepository,
    S: JobLogStreamPort,
    PS: PermissionService,
> {
    use_cases: Arc<JobUseCases<J, PS>>,
    log_use_cases: Arc<JobLogUseCases<L, PS>>,
    log_stream_use_case: Arc<JobLogStreamUseCase<L, S, PS>>,
}

#[async_trait::async_trait]
impl<
    J: JobRepository + Send + Sync + 'static,
    L: JobLogRepository + Send + Sync + 'static,
    S: JobLogStreamPort + 'static,
    PS: PermissionService + Send + Sync + 'static,
> JobService for JobHandler<J, L, S, PS>
{
    async fn get_job(
        &self,
        request: Request<GetJobRequest>,
    ) -> Result<Response<JobResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = JobId::new(&required(req.job_id, "job_id")?);

        let job = self
            .use_cases
            .get(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(job_to_proto(&job)))
    }

    async fn delete_job(
        &self,
        request: Request<DeleteJobRequest>,
    ) -> Result<Response<DeleteJobResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = JobId::new(&required(req.job_id, "job_id")?);

        self.use_cases
            .delete(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteJobResponse {}))
    }

    async fn list_jobs(
        &self,
        request: Request<ListJobsRequest>,
    ) -> Result<Response<ListJobsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(&caller, pagination.as_ref())
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
        let caller = caller!(request);
        let req = request.into_inner();
        let pipeline_id = PipelineId::new(&required(req.pipeline_id, "pipeline_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_pipeline(&caller, &pipeline_id, pagination.as_ref())
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
        let caller = caller!(request);
        let req = request.into_inner();
        let project_id = ProjectId::new(&required(req.project_id, "project_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list_by_project(&caller, &project_id, pagination.as_ref())
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

        let (jobs, metadata) = result.into_parts();
        let jobs: Vec<JobResponse> = jobs.iter().map(job_to_proto).collect();

        Ok(Response::new(ListJobsResponse {
            jobs,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_job_logs(
        &self,
        request: Request<ListJobLogsRequest>,
    ) -> Result<Response<ListJobLogsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let job_id = JobId::new(&required(req.job_id, "job_id")?);
        let pagination = proto_to_domain_pagination(req.pagination);

        let node_id_arg = optional(req.node_id);
        let result = if let Some(node_id_str) = node_id_arg.as_deref() {
            let node_id = NodeId::new(node_id_str)
                .map_err(|e| Status::invalid_argument(format!("Invalid node_id: {e}")))?;

            // Gate: log_listener and status_listener are independent broker
            // subscribers, so log rows can be persisted before the matching
            // status update reaches the recorder. Defer to the domain rule
            // before hitting the log store.
            let job = self
                .use_cases
                .get(&caller, &job_id)
                .await
                .map_err(domain_error_to_status)?;
            if !job.logs_readable_for(&node_id) {
                let params = pagination.unwrap_or_default();
                let empty_meta = PaginationMetadata::new(&params, 0);
                return Ok(Response::new(ListJobLogsResponse {
                    logs: Vec::new(),
                    pagination: Some(domain_to_proto_metadata(&empty_meta)),
                }));
            }

            self.log_use_cases
                .list_by_job_and_node(&caller, &job_id, &node_id, pagination.as_ref())
                .await
        } else {
            self.log_use_cases
                .list_by_job(&caller, &job_id, pagination.as_ref())
                .await
        }
        .map_err(domain_error_to_status)?;

        let (logs, metadata) = result.into_parts();
        let logs = logs.iter().map(job_log_to_proto).collect();

        Ok(Response::new(ListJobLogsResponse {
            logs,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    type TailJobLogsStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<JobLogEvent, Status>> + Send + 'static>>;

    async fn tail_job_logs(
        &self,
        request: Request<TailJobLogsRequest>,
    ) -> Result<Response<Self::TailJobLogsStream>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let job_id = JobId::new(&required(req.job_id, "job_id")?);
        let node_id = optional(req.node_id)
            .as_deref()
            .map(NodeId::new)
            .transpose()
            .map_err(|e| Status::invalid_argument(format!("Invalid node_id: {e}")))?;

        let stream = self
            .log_stream_use_case
            .stream(&caller, &job_id, node_id.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(Box::pin(spawn_log_forwarder(stream))))
    }
}
