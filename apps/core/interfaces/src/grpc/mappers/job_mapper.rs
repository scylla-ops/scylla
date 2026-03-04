use domain::entities::{Job, JobNode};
use protocol::services::job::{JobNodeResponse, JobResponse};

pub fn job_to_proto(job: &Job) -> JobResponse {
    JobResponse {
        job_id: job.id().to_string(),
        pipeline_id: job.pipeline_id().to_string(),
        status: job.status().as_str().to_string(),
        node_executions: job
            .node_executions()
            .iter()
            .map(job_node_to_proto)
            .collect(),
        created_at: job.created_at().to_rfc3339(),
        updated_at: job.updated_at().to_rfc3339(),
    }
}

pub fn job_node_to_proto(node: &JobNode) -> JobNodeResponse {
    JobNodeResponse {
        node_id: node.node_id().to_string(),
        state: node.state().as_str().to_string(),
        started_at: node.started_at().map(|t| t.to_rfc3339()),
        finished_at: node.finished_at().map(|t| t.to_rfc3339()),
    }
}
