use crate::application::job::GetJob;
use crate::application::pagination::PaginationMetadata;
use crate::application::{JobLogRepository, JobLogStreamPort, JobRepository};
use crate::application::{JobLogStreamUseCase, JobLogUseCases, JobUseCases};
use crate::extract_auth_context;
use crate::grpc::adapter::run;
use crate::grpc::convert::{optional, required};
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, job_log_to_proto, job_to_proto,
    proto_to_domain_pagination,
};
use crate::grpc::streaming::spawn_log_forwarder;
use derive_more::Constructor;
use scylla_auth::authz::PermissionService;
use scylla_domain::domain::ids::JobId;
use scylla_domain::domain::pipeline::NodeId;
use scylla_extension::Actions;
use scylla_proto::job::v1::{
    DeleteJobRequest, DeleteJobResponse, GetJobRequest, GetJobResponse, ListJobLogsRequest,
    ListJobLogsResponse, ListJobsRequest, ListJobsResponse, ListOrganizationJobsRequest,
    ListOrganizationJobsResponse, ListPipelineJobsRequest, ListPipelineJobsResponse,
    ListProjectJobsRequest, ListProjectJobsResponse, TailJobLogsRequest, TailJobLogsResponse,
    job_service_server::JobService,
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
    actions: Arc<Actions>,
    jobs: Arc<JobUseCases<J>>,
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
    ) -> Result<Response<GetJobResponse>, Status> {
        let job = run(&self.actions, &*self.jobs, request).await?;
        Ok(Response::new(GetJobResponse {
            job: Some(job_to_proto(&job)),
        }))
    }

    async fn delete_job(
        &self,
        request: Request<DeleteJobRequest>,
    ) -> Result<Response<DeleteJobResponse>, Status> {
        run(&self.actions, &*self.jobs, request).await?;
        Ok(Response::new(DeleteJobResponse {}))
    }

    async fn list_jobs(
        &self,
        request: Request<ListJobsRequest>,
    ) -> Result<Response<ListJobsResponse>, Status> {
        let page = run(&self.actions, &*self.jobs, request).await?;
        Ok(Response::new(page.into()))
    }

    async fn list_pipeline_jobs(
        &self,
        request: Request<ListPipelineJobsRequest>,
    ) -> Result<Response<ListPipelineJobsResponse>, Status> {
        let page = run(&self.actions, &*self.jobs, request).await?;
        Ok(Response::new(page.into()))
    }

    async fn list_project_jobs(
        &self,
        request: Request<ListProjectJobsRequest>,
    ) -> Result<Response<ListProjectJobsResponse>, Status> {
        let page = run(&self.actions, &*self.jobs, request).await?;
        Ok(Response::new(page.into()))
    }

    async fn list_organization_jobs(
        &self,
        request: Request<ListOrganizationJobsRequest>,
    ) -> Result<Response<ListOrganizationJobsResponse>, Status> {
        let page = run(&self.actions, &*self.jobs, request).await?;
        Ok(Response::new(page.into()))
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

            // Log rows can be persisted before the matching status update; the domain rule gates them.
            let job = self
                .actions
                .run(&*self.jobs, &caller, GetJob { id: job_id.clone() })
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

    type TailJobLogsStream = Pin<
        Box<dyn tokio_stream::Stream<Item = Result<TailJobLogsResponse, Status>> + Send + 'static>,
    >;

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
