use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::entities::{Job, JobNode};
use scylla_protocol::services::job::{JobNodeResponse, JobResponse};

pub fn job_to_proto(job: &Job) -> JobResponse {
    JobResponse {
        job_id: wrap(job.id().to_string()),
        pipeline_id: wrap(job.pipeline_id().to_string()),
        status: job.status().as_str().to_string(),
        node_executions: job
            .node_executions()
            .iter()
            .map(job_node_to_proto)
            .collect(),
        created_at: ts(job.created_at()),
        updated_at: ts(job.updated_at()),
        started_at: job.started_at().and_then(ts),
        finished_at: job.finished_at().and_then(ts),
    }
}

pub fn job_node_to_proto(node: &JobNode) -> JobNodeResponse {
    JobNodeResponse {
        node_id: wrap(node.node_id().to_string()),
        state: node.state().as_str().to_string(),
        started_at: node.started_at().and_then(ts),
        finished_at: node.finished_at().and_then(ts),
    }
}
