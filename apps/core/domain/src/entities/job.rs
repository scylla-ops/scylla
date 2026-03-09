use crate::entities::{JobId, Pipeline, PipelineId};
use crate::errors::{DomainError, DomainResult};
use crate::value_objects::job::{JobStatus, NodeState};
use crate::value_objects::pipeline::NodeId;
use chrono::{DateTime, Utc};

#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct JobNode {
    node_id: NodeId,
    state: NodeState,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

impl JobNode {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            state: NodeState::Pending,
            started_at: None,
            finished_at: None,
        }
    }

    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn state(&self) -> NodeState {
        self.state
    }

    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.started_at
    }

    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        self.finished_at
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct Job {
    id: JobId,
    pipeline_id: PipelineId,
    status: JobStatus,
    node_executions: Vec<JobNode>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Job {
    pub fn create_from_pipeline(pipeline: &Pipeline) -> Self {
        let now = Utc::now();

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
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_status(&mut self, new_status: JobStatus) -> DomainResult<()> {
        self.status.transition_to(&new_status)?;
        self.status = new_status;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn start(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Running)
    }

    pub fn complete(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Completed)
    }

    pub fn fail(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Failed)
    }

    pub fn cancel(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Cancelled)?;
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
            .ok_or_else(|| DomainError::validation(format!("Node not found: {}", node_id)))?;

        if exec.state != NodeState::Pending {
            return Err(DomainError::business_rule(
                "Node must be pending to start execution",
            ));
        }

        exec.state = NodeState::Running;
        exec.started_at = Some(started_at);
        self.updated_at = Utc::now();
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
                "Final state must be terminal (completed, failed, or cancelled)",
            ));
        }

        let exec = self
            .find_execution_mut(node_id)
            .ok_or_else(|| DomainError::validation(format!("Node not found: {}", node_id)))?;

        if exec.state != NodeState::Running {
            return Err(DomainError::business_rule(
                "Node must be running to finish execution",
            ));
        }

        exec.state = state;
        exec.finished_at = Some(finished_at);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn can_cancel(&self) -> bool {
        matches!(self.status, JobStatus::Pending | JobStatus::Running)
    }

    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    fn find_execution_mut(&mut self, node_id: &NodeId) -> Option<&mut JobNode> {
        self.node_executions
            .iter_mut()
            .find(|e| e.node_id() == node_id)
    }

    pub fn find_execution(&self, node_id: &NodeId) -> Option<&JobNode> {
        self.node_executions.iter().find(|e| e.node_id() == node_id)
    }

    // Getters

    pub fn id(&self) -> &JobId {
        &self.id
    }

    pub fn pipeline_id(&self) -> &PipelineId {
        &self.pipeline_id
    }

    pub fn status(&self) -> JobStatus {
        self.status
    }

    pub fn node_executions(&self) -> &[JobNode] {
        &self.node_executions
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::ProjectId;
    use crate::value_objects::pipeline::PipelineName;

    fn node_id(s: &str) -> NodeId {
        NodeId::new(s).unwrap()
    }

    fn make_pipeline(nodes: Vec<crate::entities::PipelineNode>) -> Pipeline {
        Pipeline::create(
            PipelineName::new("test").unwrap(),
            ProjectId::generate(),
            nodes,
        )
        .unwrap()
    }

    fn action(id: &str, deps: &[&str]) -> crate::entities::PipelineNode {
        crate::entities::PipelineNode::new(
            node_id(id),
            deps.iter().map(|d| node_id(d)).collect(),
            "echo".into(),
            vec![],
        )
        .unwrap()
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
        job.apply_node_started(&node_id("a"), Utc::now()).unwrap();
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

        let now = Utc::now();
        job.apply_node_started(&node_id("a"), now).unwrap();

        let exec = job.find_execution(&node_id("a")).unwrap();
        assert_eq!(exec.state(), NodeState::Running);
        assert!(exec.started_at().is_some());
    }

    #[test]
    fn apply_node_finished_sets_terminal() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        let now = Utc::now();
        job.apply_node_started(&node_id("a"), now).unwrap();
        job.apply_node_finished(&node_id("a"), NodeState::Completed, now)
            .unwrap();

        let exec = job.find_execution(&node_id("a")).unwrap();
        assert_eq!(exec.state(), NodeState::Completed);
        assert!(exec.finished_at().is_some());
    }

    #[test]
    fn cannot_finish_pending_node() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        assert!(
            job.apply_node_finished(&node_id("a"), NodeState::Completed, Utc::now())
                .is_err()
        );
    }

    #[test]
    fn cannot_start_nonexistent_node() {
        let pipeline = make_pipeline(vec![action("a", &[])]);
        let mut job = Job::create_from_pipeline(&pipeline);

        assert!(job.apply_node_started(&node_id("z"), Utc::now()).is_err());
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
