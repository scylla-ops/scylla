use crate::domain::clock;
use crate::domain::entities::{AppId, JobId, Pipeline, PipelineId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::job::{JobStatus, NodeState};
use crate::domain::value_objects::pipeline::NodeId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobNode {
    node_id: NodeId,
    state: NodeState,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

impl JobNode {
    #[must_use]
    pub fn from_persistence(
        node_id: NodeId,
        state: NodeState,
        started_at: Option<DateTime<Utc>>,
        finished_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            node_id,
            state,
            started_at,
            finished_at,
        }
    }
}

impl JobNode {
    #[must_use]
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            state: NodeState::Pending,
            started_at: None,
            finished_at: None,
        }
    }

    #[must_use]
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    #[must_use]
    pub fn state(&self) -> NodeState {
        self.state
    }

    #[must_use]
    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.started_at
    }

    #[must_use]
    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        self.finished_at
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    id: JobId,
    pipeline_id: PipelineId,
    status: JobStatus,
    node_executions: Vec<JobNode>,
    /// Trigger-supplied literal env (`(key, value)`) overlaid on every node at
    /// dispatch. Empty for a plain run. Persisted with the job so the dispatch is
    /// identical whether placed immediately or retried by the pending scheduler.
    inputs: Vec<(String, String)>,
    /// The agent (app) that executed this job, set at dispatch. `None` while
    /// pending / never dispatched.
    agent_app_id: Option<AppId>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

impl Job {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn from_persistence(
        id: JobId,
        pipeline_id: PipelineId,
        status: JobStatus,
        node_executions: Vec<JobNode>,
        inputs: Vec<(String, String)>,
        agent_app_id: Option<AppId>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        started_at: Option<DateTime<Utc>>,
        finished_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            pipeline_id,
            status,
            node_executions,
            inputs,
            agent_app_id,
            created_at,
            updated_at,
            started_at,
            finished_at,
        }
    }

    #[must_use]
    pub fn create_from_pipeline(pipeline: &Pipeline) -> Self {
        let now = clock::now();

        let node_executions: Vec<JobNode> = pipeline
            .nodes()
            .iter()
            .map(|node| JobNode::new(node.id().clone()))
            .collect();

        Self {
            id: JobId::generate(),
            pipeline_id: pipeline.id().clone(),
            status: JobStatus::Pending,
            node_executions,
            inputs: Vec::new(),
            agent_app_id: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            finished_at: None,
        }
    }

    /// Attach trigger-supplied literal inputs (builder style). These are overlaid
    /// on every node at dispatch and persisted with the job.
    #[must_use]
    pub fn with_inputs(mut self, inputs: Vec<(String, String)>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Record which agent (app) was handed this job at dispatch.
    pub fn assign_agent(&mut self, app_id: AppId) {
        self.agent_app_id = Some(app_id);
        self.updated_at = clock::now();
    }

    pub fn update_status(&mut self, new_status: JobStatus) -> DomainResult<()> {
        self.status.transition_to(&new_status)?;
        self.status = new_status;
        self.updated_at = clock::now();
        Ok(())
    }

    pub fn start(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Running)?;
        self.started_at = Some(self.updated_at);
        Ok(())
    }

    pub fn complete(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Completed)?;
        self.finished_at = Some(self.updated_at);
        Ok(())
    }

    pub fn fail(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Failed)?;
        self.finished_at = Some(self.updated_at);
        Ok(())
    }

    pub fn cancel(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Cancelled)?;
        self.finished_at = Some(self.updated_at);
        for execution in &mut self.node_executions {
            if execution.state == NodeState::Pending || execution.state == NodeState::Running {
                execution.state = NodeState::Cancelled;
            }
        }
        Ok(())
    }

    pub fn apply_node_started(
        &mut self,
        node_id: &NodeId,
        started_at: DateTime<Utc>,
    ) -> DomainResult<()> {
        let exec = self
            .find_execution_mut(node_id)
            .ok_or_else(|| DomainError::validation(format!("Node not found: {node_id}")))?;

        if exec.state != NodeState::Pending {
            return Err(DomainError::business_rule(
                "Node must be pending to start execution",
            ));
        }

        exec.state = NodeState::Running;
        exec.started_at = Some(started_at);
        self.updated_at = clock::now();
        Ok(())
    }

    pub fn apply_node_finished(
        &mut self,
        node_id: &NodeId,
        state: NodeState,
        finished_at: DateTime<Utc>,
    ) -> DomainResult<()> {
        if !state.is_terminal() {
            return Err(DomainError::business_rule(
                "Final state must be terminal (completed, failed, cancelled, or skipped)",
            ));
        }

        let exec = self
            .find_execution_mut(node_id)
            .ok_or_else(|| DomainError::validation(format!("Node not found: {node_id}")))?;

        if exec.state != NodeState::Running {
            return Err(DomainError::business_rule(
                "Node must be running to finish execution",
            ));
        }

        exec.state = state;
        exec.finished_at = Some(finished_at);
        self.updated_at = clock::now();
        Ok(())
    }

    /// Mark a node as skipped from either Pending or Running.
    ///
    /// Used when an upstream failure invalidates this node's execution.
    /// Unlike `apply_node_finished`, this accepts Pending nodes that never started.
    pub fn apply_node_skipped(
        &mut self,
        node_id: &NodeId,
        finished_at: DateTime<Utc>,
    ) -> DomainResult<()> {
        let exec = self
            .find_execution_mut(node_id)
            .ok_or_else(|| DomainError::validation(format!("Node not found: {node_id}")))?;

        if exec.state.is_terminal() {
            return Err(DomainError::business_rule(
                "Cannot skip a node already in terminal state",
            ));
        }

        exec.state = NodeState::Skipped;
        exec.finished_at = Some(finished_at);
        self.updated_at = clock::now();
        Ok(())
    }

    #[must_use]
    pub fn can_cancel(&self) -> bool {
        matches!(self.status, JobStatus::Pending | JobStatus::Running)
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
        self.status.is_terminal()
    }

    fn find_execution_mut(&mut self, node_id: &NodeId) -> Option<&mut JobNode> {
        self.node_executions
            .iter_mut()
            .find(|e| e.node_id() == node_id)
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
    pub fn status(&self) -> JobStatus {
        self.status
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

    #[must_use]
    pub fn agent_app_id(&self) -> Option<&AppId> {
        self.agent_app_id.as_ref()
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
        self.started_at
    }

    #[must_use]
    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        self.finished_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::ProjectId;
    use crate::domain::value_objects::pipeline::PipelineName;

    fn node_id(s: &str) -> NodeId {
        NodeId::new(s).unwrap()
    }

    fn make_pipeline(nodes: Vec<crate::domain::entities::PipelineNode>) -> Pipeline {
        Pipeline::create(
            PipelineName::new("test").unwrap(),
            ProjectId::generate(),
            nodes,
        )
        .unwrap()
    }

    fn action(id: &str, deps: &[&str]) -> crate::domain::entities::PipelineNode {
        use crate::domain::value_objects::pipeline::Step;
        crate::domain::entities::PipelineNode::new(
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
        let job = Job::create_from_pipeline(&pipeline);

        assert_eq!(job.status(), JobStatus::Pending);
        assert_eq!(job.node_executions().len(), 2);
        assert_eq!(job.pipeline_id(), pipeline.id());
    }

    // --- Status transitions ---

    #[test]
    fn start_transitions_to_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        assert_eq!(job.status(), JobStatus::Pending);
        job.start().unwrap();
        assert_eq!(job.status(), JobStatus::Running);
    }

    #[test]
    fn complete_transitions_from_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        job.start().unwrap();
        job.complete().unwrap();
        assert_eq!(job.status(), JobStatus::Completed);
    }

    #[test]
    fn fail_transitions_from_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        job.start().unwrap();
        job.fail().unwrap();
        assert_eq!(job.status(), JobStatus::Failed);
    }

    #[test]
    fn cancel_cancels_all_non_terminal_nodes() {
        let pipeline = make_pipeline(vec![action("a", &[]), action("b", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        job.start().unwrap();
        job.apply_node_started(&node_id("a"), clock::now()).unwrap();
        job.cancel().unwrap();

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

    // --- Node events ---

    #[test]
    fn apply_node_started_sets_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        let now = clock::now();
        job.apply_node_started(&node_id("a"), now).unwrap();

        let exec = job.find_execution(&node_id("a")).unwrap();
        assert_eq!(exec.state(), NodeState::Running);
        assert!(exec.started_at().is_some());
    }

    #[test]
    fn apply_node_finished_sets_terminal() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        let now = clock::now();
        job.apply_node_started(&node_id("a"), now).unwrap();
        job.apply_node_finished(&node_id("a"), NodeState::Completed, now)
            .unwrap();

        let exec = job.find_execution(&node_id("a")).unwrap();
        assert_eq!(exec.state(), NodeState::Completed);
        assert!(exec.finished_at().is_some());
    }

    #[test]
    fn logs_readable_for_gates_pending_nodes() {
        let pipeline = make_pipeline(vec![action("a", &[]), action("b", &["a"])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        // Both nodes start Pending → no logs leak.
        assert!(!job.logs_readable_for(&node_id("a")));
        assert!(!job.logs_readable_for(&node_id("b")));

        // Unknown node never readable.
        assert!(!job.logs_readable_for(&node_id("ghost")));

        // Running node → logs visible.
        job.apply_node_started(&node_id("a"), clock::now()).unwrap();
        assert!(job.logs_readable_for(&node_id("a")));

        // Completed node → still visible.
        job.apply_node_finished(&node_id("a"), NodeState::Completed, clock::now())
            .unwrap();
        assert!(job.logs_readable_for(&node_id("a")));

        // Sibling still Pending → still gated.
        assert!(!job.logs_readable_for(&node_id("b")));
    }

    #[test]
    fn cannot_finish_pending_node() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        assert!(
            job.apply_node_finished(&node_id("a"), NodeState::Completed, clock::now())
                .is_err()
        );
    }

    #[test]
    fn cannot_start_nonexistent_node() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        assert!(job.apply_node_started(&node_id("z"), clock::now()).is_err());
    }

    #[test]
    fn apply_node_skipped_from_pending() {
        let pipeline = make_pipeline(vec![action("a", &[]), action("b", &["a"])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        job.apply_node_skipped(&node_id("b"), clock::now()).unwrap();

        let exec = job.find_execution(&node_id("b")).unwrap();
        assert_eq!(exec.state(), NodeState::Skipped);
        assert!(exec.finished_at().is_some());
    }

    #[test]
    fn apply_node_skipped_from_running() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        job.apply_node_started(&node_id("a"), clock::now()).unwrap();
        job.apply_node_skipped(&node_id("a"), clock::now()).unwrap();

        assert_eq!(
            job.find_execution(&node_id("a")).unwrap().state(),
            NodeState::Skipped
        );
    }

    #[test]
    fn cannot_skip_terminal_node() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        let now = clock::now();
        job.apply_node_started(&node_id("a"), now).unwrap();
        job.apply_node_finished(&node_id("a"), NodeState::Completed, now)
            .unwrap();

        assert!(job.apply_node_skipped(&node_id("a"), now).is_err());
    }

    #[test]
    fn is_terminal_reflects_status() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);
        assert!(!job.is_terminal());

        job.start().unwrap();
        assert!(!job.is_terminal());

        job.complete().unwrap();
        assert!(job.is_terminal());
    }
}
