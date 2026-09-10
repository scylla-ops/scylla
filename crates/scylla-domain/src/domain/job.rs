mod event;
mod log;
mod log_stream;
mod node_state;
mod origin;
mod status;

pub use event::*;
pub use log::*;
pub use log_stream::*;
pub use node_state::*;
pub use origin::*;
pub use status::*;

use crate::domain::clock;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, JobId, PipelineId};
use crate::domain::pipeline::NodeId;
use crate::domain::pipeline::Pipeline;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// How a single pipeline node's execution has progressed. Each variant carries
/// exactly the timestamps that exist in that state — a pending node has none, a
/// running one has only its start, a finished one always has an end.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum NodeExecution {
    Pending,
    Running {
        started_at: DateTime<Utc>,
    },
    /// Reached a terminal outcome. `started_at` is `None` only when the node was
    /// skipped or cancelled before it ever ran.
    Finished {
        started_at: Option<DateTime<Utc>>,
        finished_at: DateTime<Utc>,
        outcome: NodeOutcome,
    },
}

/// The terminal outcome of a node execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeOutcome {
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

impl NodeOutcome {
    #[must_use]
    fn as_node_state(self) -> NodeState {
        match self {
            Self::Completed => NodeState::Completed,
            Self::Failed => NodeState::Failed,
            Self::Cancelled => NodeState::Cancelled,
            Self::Skipped => NodeState::Skipped,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobNode {
    node_id: NodeId,
    execution: NodeExecution,
}

impl JobNode {
    #[must_use]
    pub fn from_persistence(node_id: NodeId, execution: NodeExecution) -> Self {
        Self { node_id, execution }
    }

    #[must_use]
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            execution: NodeExecution::Pending,
        }
    }

    /// `Pending → Running`. Errs if the node has already started.
    fn start(self, started_at: DateTime<Utc>) -> DomainResult<Self> {
        match self.execution {
            NodeExecution::Pending => Ok(Self {
                node_id: self.node_id,
                execution: NodeExecution::Running { started_at },
            }),
            NodeExecution::Running { .. } | NodeExecution::Finished { .. } => Err(
                DomainError::business_rule("Node must be pending to start execution"),
            ),
        }
    }

    /// `Running → Finished` with a run outcome (completed / failed). Only a running
    /// node can finish.
    fn finish(self, outcome: NodeOutcome, finished_at: DateTime<Utc>) -> DomainResult<Self> {
        match self.execution {
            NodeExecution::Running { started_at } => Ok(Self {
                node_id: self.node_id,
                execution: NodeExecution::Finished {
                    started_at: Some(started_at),
                    finished_at,
                    outcome,
                },
            }),
            NodeExecution::Pending => Err(DomainError::business_rule(
                "Node must be running to finish execution",
            )),
            NodeExecution::Finished { .. } => {
                Err(DomainError::business_rule("Node has already finished"))
            }
        }
    }

    /// `Pending | Running → Finished(Skipped)`. Accepts a node that never started
    /// (an upstream failure invalidated it) as well as a running one.
    fn skip(self, finished_at: DateTime<Utc>) -> DomainResult<Self> {
        let started_at = match self.execution {
            NodeExecution::Pending => None,
            NodeExecution::Running { started_at } => Some(started_at),
            NodeExecution::Finished { .. } => {
                return Err(DomainError::business_rule(
                    "Cannot skip a node already in terminal state",
                ));
            }
        };
        Ok(Self {
            node_id: self.node_id,
            execution: NodeExecution::Finished {
                started_at,
                finished_at,
                outcome: NodeOutcome::Skipped,
            },
        })
    }

    /// Cancel a still-active node (called when the whole job is cancelled). A node
    /// already in a terminal state keeps its outcome.
    fn cancel_if_active(self, finished_at: DateTime<Utc>) -> Self {
        let started_at = match self.execution {
            NodeExecution::Pending => None,
            NodeExecution::Running { started_at } => Some(started_at),
            NodeExecution::Finished { .. } => return self,
        };
        Self {
            node_id: self.node_id,
            execution: NodeExecution::Finished {
                started_at,
                finished_at,
                outcome: NodeOutcome::Cancelled,
            },
        }
    }

    #[must_use]
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    #[must_use]
    pub fn execution(&self) -> &NodeExecution {
        &self.execution
    }

    /// Flat projection of the execution state, for the DB/display boundary.
    #[must_use]
    pub fn state(&self) -> NodeState {
        match &self.execution {
            NodeExecution::Pending => NodeState::Pending,
            NodeExecution::Running { .. } => NodeState::Running,
            NodeExecution::Finished { outcome, .. } => outcome.as_node_state(),
        }
    }

    #[must_use]
    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        match &self.execution {
            NodeExecution::Pending => None,
            NodeExecution::Running { started_at } => Some(*started_at),
            NodeExecution::Finished { started_at, .. } => *started_at,
        }
    }

    #[must_use]
    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        match &self.execution {
            NodeExecution::Pending | NodeExecution::Running { .. } => None,
            NodeExecution::Finished { finished_at, .. } => Some(*finished_at),
        }
    }
}

/// The terminal outcome of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalOutcome {
    Completed,
    Failed,
    Cancelled,
    Orphaned,
}

impl TerminalOutcome {
    fn from_status(status: JobStatus) -> DomainResult<Self> {
        match status {
            JobStatus::Completed => Ok(Self::Completed),
            JobStatus::Failed => Ok(Self::Failed),
            JobStatus::Cancelled => Ok(Self::Cancelled),
            JobStatus::Orphaned => Ok(Self::Orphaned),
            JobStatus::Pending | JobStatus::Running => Err(DomainError::validation(
                "non-terminal status has no terminal outcome",
            )),
        }
    }
}

/// The job lifecycle as a sum type: each state carries exactly the timestamps
/// that exist in it, so "running without a start time" or "completed without a
/// finish time" are unrepresentable.
///
/// Agent attribution is *not* part of the state — the `agent_app_id` column is a
/// nullable FK (`ON DELETE SET NULL`), so any job can lose its agent at any time
/// when the app is deleted. It is therefore an orthogonal field on [`Job`], not a
/// per-variant one.
#[derive(Debug, Clone)]
pub enum JobState {
    /// Minted, not yet started by an agent.
    Pending,
    /// The agent reported it started.
    Running { started_at: DateTime<Utc> },
    /// Ended. `finished_at` always exists; `started_at` is `None` only when the
    /// job was cancelled before it ever started running.
    Terminal {
        outcome: TerminalOutcome,
        started_at: Option<DateTime<Utc>>,
        finished_at: DateTime<Utc>,
    },
}

impl JobState {
    /// Reconstruct the state from the flat persistence columns. An inconsistent
    /// combination (a running job with no start time, a terminal one with no
    /// finish time) is a decode error, not a silently-accepted state.
    pub fn from_columns(
        status: JobStatus,
        started_at: Option<DateTime<Utc>>,
        finished_at: Option<DateTime<Utc>>,
    ) -> DomainResult<Self> {
        match status {
            JobStatus::Pending => Ok(Self::Pending),
            JobStatus::Running => Ok(Self::Running {
                started_at: started_at.ok_or_else(|| {
                    DomainError::validation("running job is missing its started_at")
                })?,
            }),
            JobStatus::Completed
            | JobStatus::Failed
            | JobStatus::Cancelled
            | JobStatus::Orphaned => Ok(Self::Terminal {
                outcome: TerminalOutcome::from_status(status)?,
                started_at,
                finished_at: finished_at.ok_or_else(|| {
                    DomainError::validation("terminal job is missing its finished_at")
                })?,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    id: JobId,
    pipeline_id: PipelineId,
    state: JobState,
    /// The agent (app) this job was dispatched to / ran on. `None` while pending
    /// and unassigned, or once the app has been deleted (nullable FK). Orthogonal
    /// to [`JobState`]; see its docs.
    agent_app_id: Option<AppId>,
    node_executions: Vec<JobNode>,
    /// Trigger-supplied literal env (`(key, value)`) overlaid on every node at
    /// dispatch. Empty for a plain run. Persisted with the job so the dispatch is
    /// identical whether placed immediately or retried by the pending scheduler.
    inputs: Vec<(String, String)>,
    /// Provenance: how this run was initiated (human / app / cron / webhook).
    /// Set at creation, immutable thereafter — a job is never unattributable.
    origin: JobOrigin,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Job {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn from_persistence(
        id: JobId,
        pipeline_id: PipelineId,
        state: JobState,
        agent_app_id: Option<AppId>,
        node_executions: Vec<JobNode>,
        inputs: Vec<(String, String)>,
        origin: JobOrigin,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            pipeline_id,
            state,
            agent_app_id,
            node_executions,
            inputs,
            origin,
            created_at,
            updated_at,
        }
    }

    /// Mint a fresh `Pending` job for `pipeline`. `origin` is mandatory — every
    /// run records who/what initiated it (see [`JobOrigin`]).
    #[must_use]
    pub fn create_from_pipeline(pipeline: &Pipeline, origin: JobOrigin) -> Self {
        let now = clock::now();

        let node_executions: Vec<JobNode> = pipeline
            .nodes()
            .iter()
            .map(|node| JobNode::new(node.id().clone()))
            .collect();

        Self {
            id: JobId::generate(),
            pipeline_id: pipeline.id().clone(),
            state: JobState::Pending,
            agent_app_id: None,
            node_executions,
            inputs: Vec::new(),
            origin,
            created_at: now,
            updated_at: now,
        }
    }

    /// Attach trigger-supplied literal inputs (builder style). These are overlaid
    /// on every node at dispatch and persisted with the job.
    #[must_use]
    pub fn with_inputs(mut self, inputs: Vec<(String, String)>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Record which agent (app) was handed this job at dispatch. Orthogonal to the
    /// lifecycle state, so it takes `&mut self` rather than transitioning it.
    pub fn assign_agent(&mut self, app_id: AppId) {
        self.agent_app_id = Some(app_id);
        self.updated_at = clock::now();
    }

    /// `Pending → Running`: the agent reported it started.
    pub fn start(mut self) -> DomainResult<Self> {
        match self.state {
            JobState::Pending => {
                let now = clock::now();
                self.state = JobState::Running { started_at: now };
                self.updated_at = now;
                Ok(self)
            }
            JobState::Running { .. } | JobState::Terminal { .. } => Err(
                DomainError::business_rule("only a pending job can start running"),
            ),
        }
    }

    /// `Running → Terminal(Completed)`.
    pub fn complete(self) -> DomainResult<Self> {
        self.finish(TerminalOutcome::Completed)
    }

    /// `Running → Terminal(Failed)`.
    pub fn fail(self) -> DomainResult<Self> {
        self.finish(TerminalOutcome::Failed)
    }

    fn finish(mut self, outcome: TerminalOutcome) -> DomainResult<Self> {
        match self.state {
            JobState::Running { started_at } => {
                let now = clock::now();
                self.state = JobState::Terminal {
                    outcome,
                    started_at: Some(started_at),
                    finished_at: now,
                };
                self.updated_at = now;
                Ok(self)
            }
            JobState::Pending | JobState::Terminal { .. } => Err(DomainError::business_rule(
                "only a running job can complete or fail",
            )),
        }
    }

    /// Cancel a non-terminal job, stamping `finished_at`. A pending job that never
    /// ran carries no `started_at`; a running one keeps its start. All still-active
    /// nodes are cancelled too.
    pub fn cancel(mut self) -> DomainResult<Self> {
        let now = clock::now();
        let started_at = match self.state {
            JobState::Pending => None,
            JobState::Running { started_at } => Some(started_at),
            JobState::Terminal { .. } => {
                return Err(DomainError::business_rule(
                    "cannot cancel a job already in a terminal state",
                ));
            }
        };
        self.state = JobState::Terminal {
            outcome: TerminalOutcome::Cancelled,
            started_at,
            finished_at: now,
        };
        for node in &mut self.node_executions {
            *node = node.clone().cancel_if_active(now);
        }
        self.updated_at = now;
        Ok(self)
    }

    /// `Pending → Running` for a single node: the agent reported the node started.
    pub fn apply_node_started(
        mut self,
        node_id: &NodeId,
        started_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        self.transition_node(node_id, |node| node.start(started_at))?;
        self.updated_at = clock::now();
        Ok(self)
    }

    /// `Running → Finished` for a single node with a run outcome.
    pub fn apply_node_finished(
        mut self,
        node_id: &NodeId,
        outcome: NodeOutcome,
        finished_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        self.transition_node(node_id, |node| node.finish(outcome, finished_at))?;
        self.updated_at = clock::now();
        Ok(self)
    }

    /// Mark a node skipped from either Pending or Running (an upstream failure
    /// invalidated it).
    pub fn apply_node_skipped(
        mut self,
        node_id: &NodeId,
        finished_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        self.transition_node(node_id, |node| node.skip(finished_at))?;
        self.updated_at = clock::now();
        Ok(self)
    }

    /// Find a node by id and replace it with the result of `transition`. The node
    /// is left untouched if the transition fails.
    fn transition_node(
        &mut self,
        node_id: &NodeId,
        transition: impl FnOnce(JobNode) -> DomainResult<JobNode>,
    ) -> DomainResult<()> {
        let index = self
            .node_executions
            .iter()
            .position(|e| e.node_id() == node_id)
            .ok_or_else(|| DomainError::validation(format!("Node not found: {node_id}")))?;
        self.node_executions[index] = transition(self.node_executions[index].clone())?;
        Ok(())
    }

    #[must_use]
    pub fn can_cancel(&self) -> bool {
        matches!(self.state, JobState::Pending | JobState::Running { .. })
    }

    /// Whether logs for `node_id` should be surfaced to readers.
    ///
    /// A node still in `Pending` has no execution behind it, so any rows in
    /// the log store that happen to be keyed to it are either stale (re-run)
    /// or arrived ahead of the matching status update — neither case should
    /// leak to the client.
    #[must_use]
    pub fn logs_readable_for(&self, node_id: &NodeId) -> bool {
        self.find_execution(node_id)
            .is_some_and(|n| n.state() != NodeState::Pending)
    }

    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, JobState::Terminal { .. })
    }

    #[must_use]
    pub fn find_execution(&self, node_id: &NodeId) -> Option<&JobNode> {
        self.node_executions.iter().find(|e| e.node_id() == node_id)
    }

    // Getters

    #[must_use]
    pub fn id(&self) -> &JobId {
        &self.id
    }

    #[must_use]
    pub fn pipeline_id(&self) -> &PipelineId {
        &self.pipeline_id
    }

    #[must_use]
    pub fn state(&self) -> &JobState {
        &self.state
    }

    /// Flat status projection, for the DB column and legacy read paths.
    #[must_use]
    pub fn status(&self) -> JobStatus {
        match &self.state {
            JobState::Pending => JobStatus::Pending,
            JobState::Running { .. } => JobStatus::Running,
            JobState::Terminal { outcome, .. } => match outcome {
                TerminalOutcome::Completed => JobStatus::Completed,
                TerminalOutcome::Failed => JobStatus::Failed,
                TerminalOutcome::Cancelled => JobStatus::Cancelled,
                TerminalOutcome::Orphaned => JobStatus::Orphaned,
            },
        }
    }

    #[must_use]
    pub fn node_executions(&self) -> &[JobNode] {
        &self.node_executions
    }

    /// Trigger-supplied literal env overlaid on every node at dispatch.
    #[must_use]
    pub fn inputs(&self) -> &[(String, String)] {
        &self.inputs
    }

    /// The agent (app) that this job was handed to / ran on, if any.
    #[must_use]
    pub fn agent_app_id(&self) -> Option<&AppId> {
        self.agent_app_id.as_ref()
    }

    /// How this run was initiated (human / app / cron / webhook).
    #[must_use]
    pub fn origin(&self) -> &JobOrigin {
        &self.origin
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[must_use]
    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        match &self.state {
            JobState::Pending => None,
            JobState::Running { started_at } => Some(*started_at),
            JobState::Terminal { started_at, .. } => *started_at,
        }
    }

    #[must_use]
    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        match &self.state {
            JobState::Pending | JobState::Running { .. } => None,
            JobState::Terminal { finished_at, .. } => Some(*finished_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::{ProjectId, UserId};
    use crate::domain::pipeline::PipelineName;

    fn node_id(s: &str) -> NodeId {
        NodeId::new(s).unwrap()
    }

    /// Mint a job from a pipeline with a throwaway human origin (these tests
    /// exercise status/node behaviour, not provenance).
    fn make_job(pipeline: &Pipeline) -> Job {
        Job::create_from_pipeline(
            pipeline,
            JobOrigin::Human {
                user_id: UserId::generate(),
            },
        )
    }

    /// A started (running) job — the common precondition for node-event and
    /// completion tests.
    fn running_job(pipeline: &Pipeline) -> Job {
        make_job(pipeline).start().unwrap()
    }

    fn make_pipeline(nodes: Vec<crate::domain::pipeline::PipelineNode>) -> Pipeline {
        Pipeline::create(
            PipelineName::new("test").unwrap(),
            ProjectId::generate(),
            nodes,
        )
        .unwrap()
    }

    fn action(id: &str, deps: &[&str]) -> crate::domain::pipeline::PipelineNode {
        use crate::domain::pipeline::Step;
        crate::domain::pipeline::PipelineNode::new(
            node_id(id),
            deps.iter().map(|d| node_id(d)).collect(),
            Step::exec("echo".into(), vec![]).unwrap(),
            None,
            vec![],
        )
    }

    // --- Creation ---

    #[test]
    fn creates_job_from_pipeline() {
        let pipeline = make_pipeline(vec![action("a", &[]), action("b", &["a"])]);
        let job = make_job(&pipeline);

        assert_eq!(job.status(), JobStatus::Pending);
        assert_eq!(job.node_executions().len(), 2);
        assert_eq!(job.pipeline_id(), pipeline.id());
        assert!(job.started_at().is_none());
    }

    // --- Status transitions ---

    #[test]
    fn start_transitions_to_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let job = make_job(&pipeline);
        assert_eq!(job.status(), JobStatus::Pending);

        let job = job.start().unwrap();
        assert_eq!(job.status(), JobStatus::Running);
        assert!(job.started_at().is_some());

        // A running job cannot start again.
        assert!(job.start().is_err());
    }

    #[test]
    fn assign_agent_is_orthogonal_to_state() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = make_job(&pipeline);
        assert!(job.agent_app_id().is_none());
        job.assign_agent(AppId::generate());
        assert!(job.agent_app_id().is_some());
        assert_eq!(job.status(), JobStatus::Pending);
    }

    #[test]
    fn complete_transitions_from_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let job = running_job(&pipeline).complete().unwrap();
        assert_eq!(job.status(), JobStatus::Completed);
        assert!(job.finished_at().is_some());
    }

    #[test]
    fn fail_transitions_from_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let job = running_job(&pipeline).fail().unwrap();
        assert_eq!(job.status(), JobStatus::Failed);
        assert!(job.finished_at().is_some());
    }

    #[test]
    fn cannot_complete_a_pending_job() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        assert!(make_job(&pipeline).complete().is_err());
    }

    #[test]
    fn cancel_cancels_all_non_terminal_nodes() {
        let pipeline = make_pipeline(vec![action("a", &[]), action("b", &[])]);
        let job = running_job(&pipeline)
            .apply_node_started(&node_id("a"), clock::now())
            .unwrap()
            .cancel()
            .unwrap();

        assert_eq!(job.status(), JobStatus::Cancelled);
        assert_eq!(
            job.find_execution(&node_id("a")).unwrap().state(),
            NodeState::Cancelled
        );
        assert_eq!(
            job.find_execution(&node_id("b")).unwrap().state(),
            NodeState::Cancelled
        );
    }

    #[test]
    fn cancel_from_pending_has_no_start_time() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let job = make_job(&pipeline).cancel().unwrap();
        assert_eq!(job.status(), JobStatus::Cancelled);
        assert!(job.started_at().is_none());
        assert!(job.finished_at().is_some());
    }

    // --- Node events ---

    #[test]
    fn apply_node_started_sets_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let now = clock::now();
        let job = running_job(&pipeline)
            .apply_node_started(&node_id("a"), now)
            .unwrap();

        let exec = job.find_execution(&node_id("a")).unwrap();
        assert_eq!(exec.state(), NodeState::Running);
        assert!(exec.started_at().is_some());
    }

    #[test]
    fn apply_node_finished_sets_terminal() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let now = clock::now();
        let job = running_job(&pipeline)
            .apply_node_started(&node_id("a"), now)
            .unwrap()
            .apply_node_finished(&node_id("a"), NodeOutcome::Completed, now)
            .unwrap();

        let exec = job.find_execution(&node_id("a")).unwrap();
        assert_eq!(exec.state(), NodeState::Completed);
        assert!(exec.finished_at().is_some());
    }

    #[test]
    fn logs_readable_for_gates_pending_nodes() {
        let pipeline = make_pipeline(vec![action("a", &[]), action("b", &["a"])]);
        let mut job = running_job(&pipeline);

        // Both nodes start Pending → no logs leak.
        assert!(!job.logs_readable_for(&node_id("a")));
        assert!(!job.logs_readable_for(&node_id("b")));

        // Unknown node never readable.
        assert!(!job.logs_readable_for(&node_id("ghost")));

        // Running node → logs visible.
        job = job.apply_node_started(&node_id("a"), clock::now()).unwrap();
        assert!(job.logs_readable_for(&node_id("a")));

        // Completed node → still visible.
        job = job
            .apply_node_finished(&node_id("a"), NodeOutcome::Completed, clock::now())
            .unwrap();
        assert!(job.logs_readable_for(&node_id("a")));

        // Sibling still Pending → still gated.
        assert!(!job.logs_readable_for(&node_id("b")));
    }

    #[test]
    fn cannot_finish_pending_node() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let job = running_job(&pipeline);

        assert!(
            job.apply_node_finished(&node_id("a"), NodeOutcome::Completed, clock::now())
                .is_err()
        );
    }

    #[test]
    fn cannot_start_nonexistent_node() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let job = running_job(&pipeline);

        assert!(job.apply_node_started(&node_id("z"), clock::now()).is_err());
    }

    #[test]
    fn apply_node_skipped_from_pending() {
        let pipeline = make_pipeline(vec![action("a", &[]), action("b", &["a"])]);
        let job = running_job(&pipeline)
            .apply_node_skipped(&node_id("b"), clock::now())
            .unwrap();

        let exec = job.find_execution(&node_id("b")).unwrap();
        assert_eq!(exec.state(), NodeState::Skipped);
        assert!(exec.finished_at().is_some());
    }

    #[test]
    fn apply_node_skipped_from_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let job = running_job(&pipeline)
            .apply_node_started(&node_id("a"), clock::now())
            .unwrap()
            .apply_node_skipped(&node_id("a"), clock::now())
            .unwrap();

        assert_eq!(
            job.find_execution(&node_id("a")).unwrap().state(),
            NodeState::Skipped
        );
    }

    #[test]
    fn cannot_skip_terminal_node() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let now = clock::now();
        let job = running_job(&pipeline)
            .apply_node_started(&node_id("a"), now)
            .unwrap()
            .apply_node_finished(&node_id("a"), NodeOutcome::Completed, now)
            .unwrap();

        assert!(job.apply_node_skipped(&node_id("a"), now).is_err());
    }

    #[test]
    fn is_terminal_reflects_status() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let job = make_job(&pipeline);
        assert!(!job.is_terminal());

        let job = job.start().unwrap();
        assert!(!job.is_terminal());

        let job = job.complete().unwrap();
        assert!(job.is_terminal());
    }

    #[test]
    fn cannot_cancel_terminal_job() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let job = running_job(&pipeline).complete().unwrap();
        assert!(!job.can_cancel());
        assert!(job.cancel().is_err());
    }

    /// Golden test for the `jobs.node_executions` JSONB column.
    ///
    /// `NodeExecution` is an internally tagged enum, so its variant names are
    /// part of the on-disk format, not just an implementation detail. Renaming
    /// a variant or a field would silently orphan the execution history of every
    /// job already recorded in a deployed database. See the equivalent test on
    /// `PipelineNode` for the same guarantee on `pipelines.nodes`.
    #[test]
    fn job_nodes_jsonb_shape_is_stable() {
        const STORED: &str = r#"[
            {"node_id":"a","execution":{"state":"pending"}},
            {"node_id":"b","execution":{"state":"running","started_at":"2026-01-15T10:30:00Z"}},
            {"node_id":"c","execution":{"state":"finished","started_at":"2026-01-15T10:30:00Z",
             "finished_at":"2026-01-15T10:30:00Z","outcome":"completed"}},
            {"node_id":"d","execution":{"state":"finished","started_at":null,
             "finished_at":"2026-01-15T10:30:00Z","outcome":"skipped"}}
        ]"#;

        let nodes: Vec<JobNode> = serde_json::from_str(STORED).unwrap();
        assert_eq!(nodes.len(), 4);
        assert!(matches!(nodes[0].execution(), NodeExecution::Pending));
        assert!(matches!(
            nodes[1].execution(),
            NodeExecution::Running { .. }
        ));
        assert!(matches!(
            nodes[2].execution(),
            NodeExecution::Finished {
                outcome: NodeOutcome::Completed,
                started_at: Some(_),
                ..
            }
        ));
        // A node skipped before it ever ran has no start timestamp.
        assert!(matches!(
            nodes[3].execution(),
            NodeExecution::Finished {
                outcome: NodeOutcome::Skipped,
                started_at: None,
                ..
            }
        ));

        let round_tripped: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&nodes).unwrap()).unwrap();
        let original: serde_json::Value = serde_json::from_str(STORED).unwrap();
        assert_eq!(round_tripped, original, "serialized shape drifted");
    }
}
