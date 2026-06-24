use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::entities::{Job, JobNode};
use scylla_core::domain::value_objects::job::JobOrigin;
use scylla_protocol::services::job::{
    AppOrigin, CronOrigin, HumanOrigin, JobNodeResponse, JobResponse, WebhookOrigin, job_response,
};

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
        origin: Some(origin_to_proto(job.origin())),
    }
}

/// Map the domain provenance to the `JobResponse.origin` oneof (mirrors the
/// trigger handler's `source_to_proto`).
fn origin_to_proto(origin: &JobOrigin) -> job_response::Origin {
    match origin {
        JobOrigin::Human { user_id } => job_response::Origin::Human(HumanOrigin {
            user_id: user_id.to_string(),
        }),
        JobOrigin::App { app_id } => job_response::Origin::App(AppOrigin {
            app_id: app_id.to_string(),
        }),
        JobOrigin::Cron { trigger_id } => job_response::Origin::Cron(CronOrigin {
            trigger_id: trigger_id.to_string(),
        }),
        JobOrigin::Webhook {
            trigger_id,
            delivery_id,
        } => job_response::Origin::Webhook(WebhookOrigin {
            trigger_id: trigger_id.to_string(),
            delivery_id: delivery_id.clone().unwrap_or_default(),
        }),
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
