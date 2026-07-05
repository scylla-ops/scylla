use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::entities::{
    Job, JobNode, JobState, NodeExecution, NodeOutcome, TerminalOutcome,
};
use scylla_core::domain::value_objects::job::JobOrigin;
use scylla_protocol::services::job::{
    AppOrigin, CronOrigin, HumanOrigin, JobNodeResponse, JobOutcome, JobPending, JobResponse,
    JobRunning, JobTerminal, NodeFinished, NodeOutcome as ProtoNodeOutcome, NodePending,
    NodeRunning, WebhookOrigin, job_node_response, job_response,
};

pub fn job_to_proto(job: &Job) -> JobResponse {
    JobResponse {
        job_id: wrap(job.id().to_string()),
        pipeline_id: wrap(job.pipeline_id().to_string()),
        node_executions: job
            .node_executions()
            .iter()
            .map(job_node_to_proto)
            .collect(),
        created_at: ts(job.created_at()),
        updated_at: ts(job.updated_at()),
        state: Some(job_state_to_proto(job.state())),
        origin: Some(origin_to_proto(job.origin())),
    }
}

/// Map the domain lifecycle to the `JobResponse.state` oneof — each arm builds
/// exactly the variant that carries this state's timestamps.
fn job_state_to_proto(state: &JobState) -> job_response::State {
    match state {
        JobState::Pending => job_response::State::Pending(JobPending {}),
        JobState::Running { started_at } => job_response::State::Running(JobRunning {
            started_at: ts(*started_at),
        }),
        JobState::Terminal {
            outcome,
            started_at,
            finished_at,
        } => job_response::State::Terminal(JobTerminal {
            outcome: job_outcome_to_proto(*outcome) as i32,
            started_at: started_at.and_then(ts),
            finished_at: ts(*finished_at),
        }),
    }
}

fn job_outcome_to_proto(outcome: TerminalOutcome) -> JobOutcome {
    match outcome {
        TerminalOutcome::Completed => JobOutcome::JobCompleted,
        TerminalOutcome::Failed => JobOutcome::JobFailed,
        TerminalOutcome::Cancelled => JobOutcome::JobCancelled,
        TerminalOutcome::Orphaned => JobOutcome::JobOrphaned,
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
        execution: Some(node_execution_to_proto(node.execution())),
    }
}

fn node_execution_to_proto(execution: &NodeExecution) -> job_node_response::Execution {
    match execution {
        NodeExecution::Pending => job_node_response::Execution::Pending(NodePending {}),
        NodeExecution::Running { started_at } => {
            job_node_response::Execution::Running(NodeRunning {
                started_at: ts(*started_at),
            })
        }
        NodeExecution::Finished {
            started_at,
            finished_at,
            outcome,
        } => job_node_response::Execution::Finished(NodeFinished {
            outcome: node_outcome_to_proto(*outcome) as i32,
            started_at: started_at.and_then(ts),
            finished_at: ts(*finished_at),
        }),
    }
}

fn node_outcome_to_proto(outcome: NodeOutcome) -> ProtoNodeOutcome {
    match outcome {
        NodeOutcome::Completed => ProtoNodeOutcome::NodeCompleted,
        NodeOutcome::Failed => ProtoNodeOutcome::NodeFailed,
        NodeOutcome::Cancelled => ProtoNodeOutcome::NodeCancelled,
        NodeOutcome::Skipped => ProtoNodeOutcome::NodeSkipped,
    }
}
