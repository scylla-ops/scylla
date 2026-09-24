//! Wire to command, command outcome to wire. The handler holds none of it.

use crate::application::job::{
    DeleteJob, GetJob, ListJobs, ListOrganizationJobs, ListPipelineJobs, ListProjectJobs,
};
use crate::grpc::convert::{ts, wrap};
use scylla_domain::domain::job::JobOrigin;
use scylla_domain::domain::job::{
    Job, JobNode, JobState, NodeExecution, NodeOutcome, TerminalOutcome,
};
use scylla_proto::job::v1::{
    DeleteJobRequest, GetJobRequest, Job as ProtoJob, JobNode as ProtoJobNode, JobOutcome,
    ListJobsRequest, ListJobsResponse, ListOrganizationJobsRequest, ListOrganizationJobsResponse,
    ListPipelineJobsRequest, ListPipelineJobsResponse, ListProjectJobsRequest,
    ListProjectJobsResponse, NodeOutcome as ProtoNodeOutcome, job, job_node,
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

parse!(GetJobRequest => GetJob { id: id(job_id) });
parse!(DeleteJobRequest => DeleteJob { id: id(job_id) });
parse!(ListJobsRequest => ListJobs { pagination: page });

parse!(ListPipelineJobsRequest => ListPipelineJobs {
    pipeline_id: id(pipeline_id),
    pagination: page,
});

parse!(ListProjectJobsRequest => ListProjectJobs {
    project_id: id(project_id),
    pagination: page,
});

parse!(ListOrganizationJobsRequest => ListOrganizationJobs {
    organization_id: id(organization_id),
    pagination: page,
});

page_response!(
    Job => jobs: job_to_proto;
    ListJobsResponse,
    ListPipelineJobsResponse,
    ListProjectJobsResponse,
    ListOrganizationJobsResponse,
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grpc::convert::Parse;
    use scylla_proto::common::v1 as common;
    use tonic::Code;

    #[test]
    fn a_get_request_becomes_a_query_on_the_job() {
        let query = GetJobRequest {
            job_id: wrap("job-1"),
        }
        .parse()
        .unwrap();

        assert_eq!(query.id.as_str(), "job-1");
    }

    #[test]
    fn a_missing_id_is_an_invalid_argument() {
        let err = DeleteJobRequest {
            job_id: None::<common::JobId>,
        }
        .parse()
        .unwrap_err();

        assert_eq!(err.code(), Code::InvalidArgument);
        assert_eq!(err.message(), "missing job_id");
    }

    #[test]
    fn a_pipeline_list_request_carries_its_pipeline_and_page() {
        let query = ListPipelineJobsRequest {
            pipeline_id: wrap("pipe-1"),
            pagination: None,
        }
        .parse()
        .unwrap();

        assert_eq!(query.pipeline_id.as_str(), "pipe-1");
        assert!(query.pagination.is_none());
    }
}
