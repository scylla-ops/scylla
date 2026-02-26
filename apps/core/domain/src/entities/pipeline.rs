use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{NodeId, NodeName, PipelineId};
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// PipelineNode represents a node (action or group) in a pipeline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineNode {
    /// A group node (logical grouping of actions)
    Group {
        id: NodeId, //
        name: NodeName, //build
        deps: Vec<NodeId>,
    },
    /// An action node (executable task)
    Action {
        id: NodeId,
        name: NodeName,
        deps: Vec<NodeId>,
        command: String,
        args: Vec<String>,
    },
}

impl PipelineNode {
    /// Get the node's ID
    pub fn id(&self) -> &NodeId {
        match self {
            PipelineNode::Group { id, .. } | PipelineNode::Action { id, .. } => id,
        }
    }

    /// Get the node's name
    pub fn name(&self) -> &NodeName {
        match self {
            PipelineNode::Group { name, .. } | PipelineNode::Action { name, .. } => name,
        }
    }

    /// Get the node's dependencies
    pub fn deps(&self) -> &[NodeId] {
        match self {
            PipelineNode::Group { deps, .. } | PipelineNode::Action { deps, .. } => deps,
        }
    }

    /// Validate that all dependencies exist in the pipeline
    fn validate_deps(&self, all_ids: &[NodeId]) -> DomainResult<()> {
        for dep_id in self.deps() {
            if !all_ids.contains(dep_id) {
                return Err(DomainError::validation(format!(
                    "Invalid dependency: {}",
                    dep_id
                )));
            }
        }
        Ok(())
    }
}

/// Pipeline domain entity - represents the definition of a pipeline
#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct Pipeline {
    id: PipelineId,
    name: String,
    nodes: Vec<PipelineNode>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Pipeline {
    /// Create a new pipeline with validation
    pub fn create(name: impl Into<String>, nodes: Vec<PipelineNode>) -> DomainResult<Self> {
        let name = name.into();
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(DomainError::validation("Pipeline name cannot be empty"));
        }

        if nodes.is_empty() {
            return Err(DomainError::validation(
                "Pipeline must have at least one node",
            ));
        }

        let all_ids: Vec<NodeId> = nodes.iter().map(|n| n.id().clone()).collect();

        // Validate all dependencies
        for node in &nodes {
            node.validate_deps(&all_ids)?;
        }

        // Validate no cycles
        Self::validate_no_cycles(&nodes)?;

        let now = Utc::now();
        Ok(Pipeline {
            id: PipelineId::generate(),
            name: trimmed.to_string(),
            nodes,
            created_at: now,
            updated_at: now,
        })
    }

    /// Validate that there are no cycles in the pipeline DAG
    fn validate_no_cycles(nodes: &[PipelineNode]) -> DomainResult<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in nodes {
            if !visited.contains(node.id()) {
                Self::dfs(node.id(), nodes, &mut visited, &mut rec_stack)?;
            }
        }

        Ok(())
    }

    /// Depth-first search to detect cycles
    fn dfs(
        node_id: &NodeId,
        nodes: &[PipelineNode],
        visited: &mut HashSet<NodeId>,
        rec_stack: &mut HashSet<NodeId>,
    ) -> DomainResult<()> {
        visited.insert(node_id.clone());
        rec_stack.insert(node_id.clone());

        let node = nodes
            .iter()
            .find(|n| n.id() == node_id)
            .ok_or(DomainError::validation("Node not found"))?;

        for dep in node.deps() {
            if !visited.contains(dep) {
                Self::dfs(dep, nodes, visited, rec_stack)?;
            } else if rec_stack.contains(dep) {
                return Err(DomainError::business_rule("Cycle detected in pipeline"));
            }
        }

        rec_stack.remove(node_id);
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &PipelineId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn nodes(&self) -> &[PipelineNode] {
        &self.nodes
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Get the dependencies of a specific node
    pub fn get_node_dependencies(&self, node_id: &NodeId) -> Option<Vec<NodeId>> {
        self.nodes
            .iter()
            .find(|n| n.id() == node_id)
            .map(|n| n.deps().to_vec())
    }
}
