use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::job::JobOrigin;
use scylla_core::domain::job::{
    Job, JobNode, JobState, NodeExecution, NodeOutcome, TerminalOutcome,
};
use scylla_protocol::job::v1::{
    Job as ProtoJob, JobNode as ProtoJobNode, JobOutcome, NodeOutcome as ProtoNodeOutcome, job,
    job_node,
};

pub fn job_to_proto(job: &Job) -> ProtoJob {
    ProtoJob {
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

/// Map the domain lifecycle to the `Job.state` oneof — each arm builds
/// exactly the variant that carries this state's timestamps.
fn job_state_to_proto(state: &JobState) -> job::State {
    match state {
        JobState::Pending => job::State::Pending(job::Pending {}),
        JobState::Running { started_at } => job::State::Running(job::Running {
            started_at: ts(*started_at),
        }),
        JobState::Terminal {
            outcome,
            started_at,
            finished_at,
        } => job::State::Terminal(job::Terminal {
            outcome: job_outcome_to_proto(*outcome) as i32,
            started_at: started_at.and_then(ts),
            finished_at: ts(*finished_at),
        }),
    }
}

fn job_outcome_to_proto(outcome: TerminalOutcome) -> JobOutcome {
    match outcome {
        TerminalOutcome::Completed => JobOutcome::Completed,
        TerminalOutcome::Failed => JobOutcome::Failed,
        TerminalOutcome::Cancelled => JobOutcome::Cancelled,
        TerminalOutcome::Orphaned => JobOutcome::Orphaned,
    }
}

/// Map the domain provenance to the `Job.origin` oneof (mirrors the
/// trigger handler's `source_to_proto`).
fn origin_to_proto(origin: &JobOrigin) -> job::Origin {
    match origin {
        JobOrigin::Human { user_id } => job::Origin::Human(job::Human {
            user_id: wrap(user_id.to_string()),
        }),
        JobOrigin::App { app_id } => job::Origin::App(job::App {
            app_id: wrap(app_id.to_string()),
        }),
        JobOrigin::Cron { trigger_id } => job::Origin::Cron(job::Cron {
            trigger_id: wrap(trigger_id.to_string()),
        }),
        JobOrigin::Webhook {
            trigger_id,
            delivery_id,
        } => job::Origin::Webhook(job::Webhook {
            trigger_id: wrap(trigger_id.to_string()),
            delivery_id: delivery_id.clone(),
        }),
    }
}

pub fn job_node_to_proto(node: &JobNode) -> ProtoJobNode {
    ProtoJobNode {
        node_id: wrap(node.node_id().to_string()),
        execution: Some(node_execution_to_proto(node.execution())),
    }
}

fn node_execution_to_proto(execution: &NodeExecution) -> job_node::Execution {
    match execution {
        NodeExecution::Pending => job_node::Execution::Pending(job_node::Pending {}),
        NodeExecution::Running { started_at } => job_node::Execution::Running(job_node::Running {
            started_at: ts(*started_at),
        }),
        NodeExecution::Finished {
            started_at,
            finished_at,
            outcome,
        } => job_node::Execution::Finished(job_node::Finished {
            outcome: node_outcome_to_proto(*outcome) as i32,
            started_at: started_at.and_then(ts),
            finished_at: ts(*finished_at),
        }),
    }
}

fn node_outcome_to_proto(outcome: NodeOutcome) -> ProtoNodeOutcome {
    match outcome {
        NodeOutcome::Completed => ProtoNodeOutcome::Completed,
        NodeOutcome::Failed => ProtoNodeOutcome::Failed,
        NodeOutcome::Cancelled => ProtoNodeOutcome::Cancelled,
        NodeOutcome::Skipped => ProtoNodeOutcome::Skipped,
    }
}
