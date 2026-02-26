use crate::domain::entities::pipeline::Pipeline;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{JobId, JobStatus, NodeId, PipelineId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JobNodeExecution tracks the execution state of a node within a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobNodeExecution {
    node_id: NodeId,
    state: JobStatus,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

impl JobNodeExecution {
    /// Create a new node execution
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            state: JobStatus::Pending,
            started_at: None,
            finished_at: None,
        }
    }

    /// Start the node execution
    pub fn start(&mut self) -> DomainResult<()> {
        if self.state != JobStatus::Pending {
            return Err(DomainError::business_rule(
                "Node must be pending to start execution",
            ));
        }
        self.state = JobStatus::Running;
        self.started_at = Some(Utc::now());
        Ok(())
    }

    /// Complete the node execution with a final state
    pub fn finish(&mut self, state: JobStatus) -> DomainResult<()> {
        if self.state != JobStatus::Running {
            return Err(DomainError::business_rule(
                "Node must be running to finish execution",
            ));
        }

        if state == JobStatus::Pending || state == JobStatus::Running {
            return Err(DomainError::business_rule(
                "Final state cannot be pending or running",
            ));
        }

        self.state = state;
        self.finished_at = Some(Utc::now());
        Ok(())
    }

    // Getters
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn state(&self) -> JobStatus {
        self.state
    }

    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.started_at
    }

    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        self.finished_at
    }
}

/// Job domain entity - represents an instance of a pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    id: JobId,
    pipeline_id: PipelineId,
    status: JobStatus,
    executions: HashMap<NodeId, JobNodeExecution>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Job {
    /// Create a new job from a pipeline
    pub fn create_from_pipeline(pipeline: &Pipeline) -> DomainResult<Self> {
        let now = Utc::now();

        let mut executions = HashMap::new();
        for node in pipeline.nodes() {
            executions.insert(node.id().clone(), JobNodeExecution::new(node.id().clone()));
        }

        Ok(Self {
            id: JobId::generate(),
            pipeline_id: PipelineId::from(pipeline.id()),
            status: JobStatus::Pending,
            executions,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update job status with validation
    pub fn update_status(&mut self, new_status: JobStatus) -> DomainResult<()> {
        self.status.validate_transition_to(&new_status)?;
        self.status = new_status;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Start the job (transition from Pending to Running)
    pub fn start(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Running)
    }

    /// Mark job as completed
    pub fn complete(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Completed)
    }

    /// Mark job as failed
    pub fn fail(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Failed)
    }

    /// Cancel the job
    pub fn cancel(&mut self) -> DomainResult<()> {
        self.update_status(JobStatus::Cancelled)
    }

    /// Check if job can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(self.status, JobStatus::Pending | JobStatus::Running)
    }

    /// Start a node execution within the job
    pub fn start_node(&mut self, node_id: &NodeId) -> DomainResult<()> {
        if !matches!(self.status, JobStatus::Pending | JobStatus::Running) {
            return Err(DomainError::business_rule(
                "Cannot execute nodes in a terminal job",
            ));
        }

        if self.status == JobStatus::Pending {
            self.start()?;
        }

        if let Some(execution) = self.executions.get_mut(node_id) {
            execution.start()?;
            Ok(())
        } else {
            Err(DomainError::validation(format!(
                "Node not found: {}",
                node_id
            )))
        }
    }

    /// Finish a node execution within the job
    pub fn finish_node(&mut self, node_id: &NodeId, state: JobStatus) -> DomainResult<()> {
        if let Some(execution) = self.executions.get_mut(node_id) {
            execution.finish(state)?;

            // Update job status based on node results
            if state == JobStatus::Failed {
                self.fail()?;
            } else if self
                .executions
                .values()
                .all(|n| n.state() == JobStatus::Completed)
            {
                self.complete()?;
            }

            Ok(())
        } else {
            Err(DomainError::validation(format!(
                "Node not found: {}",
                node_id
            )))
        }
    }

    /// Get nodes that are ready to be executed
    pub fn runnable_nodes(&self, pipeline: &Pipeline) -> Vec<&NodeId> {
        self.executions
            .values()
            .filter(|execution| {
                execution.state() == JobStatus::Pending
                    && pipeline
                        .get_node_dependencies(execution.node_id())
                        .map(|deps| {
                            deps.iter().all(|dep_id| {
                                self.executions
                                    .get(dep_id)
                                    .map(|dep_exec| dep_exec.state() == JobStatus::Completed)
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(true)
            })
            .map(|e| e.node_id())
            .collect()
    }

    // Getters
    pub fn id(&self) -> &JobId {
        &self.id
    }

    pub fn pipeline_id(&self) -> &PipelineId {
        &self.pipeline_id
    }

    pub fn status(&self) -> &JobStatus {
        &self.status
    }

    pub fn executions(&self) -> &HashMap<NodeId, JobNodeExecution> {
        &self.executions
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
}
