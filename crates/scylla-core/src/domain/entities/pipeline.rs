use crate::domain::clock;
use crate::domain::entities::{PipelineId, ProjectId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::pipeline::{EnvVar, NodeId, PipelineName, Step, WorkingDir};
use chrono::{DateTime, Utc};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineNode {
    id: NodeId,
    deps: Vec<NodeId>,
    /// Working directory for the step, relative to the per-job workspace root.
    /// `None` runs in the workspace root.
    #[serde(default)]
    working_dir: Option<WorkingDir>,
    /// Node-scoped environment overlay (literal values).
    #[serde(default)]
    env: Vec<EnvVar>,
    /// What the node runs: a direct exec or a shell script.
    step: Step,
}

impl PipelineNode {
    /// Assemble a node. The `step`, `working_dir`, and `env` are already
    /// validated by their own constructors/types, so this only wires them
    /// together; DAG-level checks live in [`Pipeline::create`].
    #[must_use]
    pub fn new(
        id: NodeId,
        deps: Vec<NodeId>,
        step: Step,
        working_dir: Option<WorkingDir>,
        env: Vec<EnvVar>,
    ) -> Self {
        Self {
            id,
            deps,
            working_dir,
            env,
            step,
        }
    }

    #[must_use]
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    #[must_use]
    pub fn deps(&self) -> &[NodeId] {
        &self.deps
    }

    #[must_use]
    pub fn step(&self) -> &Step {
        &self.step
    }

    #[must_use]
    pub fn working_dir(&self) -> Option<&WorkingDir> {
        self.working_dir.as_ref()
    }

    #[must_use]
    pub fn env(&self) -> &[EnvVar] {
        &self.env
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    id: PipelineId,
    project_id: ProjectId,
    name: PipelineName,
    nodes: Vec<PipelineNode>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Pipeline {
    /// Reconstitute a `Pipeline` from persistent storage without re-running
    /// DAG validation. Bypassing validation is safe here because nodes were
    /// validated at create/update time and JSONB is round-tripped verbatim.
    #[must_use]
    pub fn from_persistence(
        id: PipelineId,
        project_id: ProjectId,
        name: PipelineName,
        nodes: Vec<PipelineNode>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            nodes,
            created_at,
            updated_at,
        }
    }

    pub fn create(
        name: PipelineName,
        project_id: ProjectId,
        nodes: Vec<PipelineNode>,
    ) -> DomainResult<Self> {
        Self::validate_nodes(&nodes)?;

        let now = clock::now();
        Ok(Pipeline {
            id: PipelineId::generate(),
            project_id,
            name,
            nodes,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_name(&mut self, name: PipelineName) -> DomainResult<()> {
        self.name = name;
        self.updated_at = clock::now();
        Ok(())
    }

    pub fn update_nodes(&mut self, nodes: Vec<PipelineNode>) -> DomainResult<()> {
        Self::validate_nodes(&nodes)?;
        self.nodes = nodes;
        self.updated_at = clock::now();
        Ok(())
    }

    fn validate_nodes(nodes: &[PipelineNode]) -> DomainResult<()> {
        if nodes.is_empty() {
            return Err(DomainError::validation(
                "Pipeline must have at least one node",
            ));
        }

        let mut node_ids = HashSet::new();
        for node in nodes {
            if !node_ids.insert(node.id()) {
                return Err(DomainError::validation(format!(
                    "Duplicate node ID: {}",
                    node.id()
                )));
            }
        }

        for node in nodes {
            let mut seen_deps = HashSet::new();
            for dep_id in node.deps() {
                if dep_id == node.id() {
                    return Err(DomainError::validation(format!(
                        "Node '{}' cannot depend on itself",
                        node.id()
                    )));
                }
                if !node_ids.contains(dep_id) {
                    return Err(DomainError::validation(format!(
                        "Node '{}' has invalid dependency: {}",
                        node.id(),
                        dep_id
                    )));
                }
                if !seen_deps.insert(dep_id) {
                    return Err(DomainError::validation(format!(
                        "Node '{}' has duplicate dependency: {}",
                        node.id(),
                        dep_id
                    )));
                }
            }
        }

        // cycle detection, Kahn's algorithm
        let mut in_degree: HashMap<&NodeId, usize> =
            nodes.iter().map(|n| (n.id(), n.deps().len())).collect();

        let mut adjacency: HashMap<&NodeId, Vec<&NodeId>> =
            nodes.iter().map(|n| (n.id(), Vec::new())).collect();
        for node in nodes {
            for dep_id in node.deps() {
                adjacency.get_mut(dep_id).unwrap().push(node.id());
            }
        }

        let mut ready: BTreeSet<&NodeId> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(id, _)| *id)
            .collect();

        let mut visited = 0usize;

        while let Some(current) = ready.pop_first() {
            visited += 1;
            if let Some(dependents) = adjacency.get(current) {
                for dependent in dependents {
                    let deg = in_degree.get_mut(dependent).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        ready.insert(dependent);
                    }
                }
            }
        }

        if visited != nodes.len() {
            return Err(DomainError::business_rule("Cycle detected in pipeline"));
        }

        Ok(())
    }

    #[must_use]
    pub fn id(&self) -> &PipelineId {
        &self.id
    }

    #[must_use]
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    #[must_use]
    pub fn name(&self) -> &PipelineName {
        &self.name
    }

    #[must_use]
    pub fn nodes(&self) -> &[PipelineNode] {
        &self.nodes
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
    pub fn get_node(&self, node_id: &NodeId) -> Option<&PipelineNode> {
        self.nodes.iter().find(|n| n.id() == node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::pipeline::Step;

    fn node_id(s: &str) -> NodeId {
        NodeId::new(s).unwrap()
    }

    fn pipeline_name() -> PipelineName {
        PipelineName::new("test-pipeline").unwrap()
    }

    fn project_id() -> ProjectId {
        ProjectId::generate()
    }

    fn action(id: &str, deps: &[&str]) -> PipelineNode {
        PipelineNode::new(
            node_id(id),
            deps.iter().map(|d| node_id(d)).collect(),
            Step::exec("echo".into(), vec![]).unwrap(),
            None,
            vec![],
        )
    }

    #[test]
    fn valid_dag() {
        let nodes = vec![
            action("a", &[]),
            action("b", &["a"]),
            action("c", &["a", "b"]),
        ];
        let pipeline = Pipeline::create(pipeline_name(), project_id(), nodes).unwrap();
        assert_eq!(pipeline.nodes().len(), 3);
    }

    #[test]
    fn get_node_returns_correct_node() {
        let nodes = vec![action("a", &[]), action("b", &["a"])];
        let pipeline = Pipeline::create(pipeline_name(), project_id(), nodes).unwrap();
        let node = pipeline.get_node(&node_id("a")).unwrap();
        assert_eq!(node.id(), &node_id("a"));
        assert!(pipeline.get_node(&node_id("z")).is_none());
    }

    #[test]
    fn rejects_cycle() {
        let nodes = vec![
            action("a", &["c"]),
            action("b", &["a"]),
            action("c", &["b"]),
        ];
        assert!(Pipeline::create(pipeline_name(), project_id(), nodes).is_err());
    }

    #[test]
    fn rejects_invalid_dependency() {
        let nodes = vec![action("a", &["nonexistent"])];
        assert!(Pipeline::create(pipeline_name(), project_id(), nodes).is_err());
    }

    #[test]
    fn rejects_duplicate_ids() {
        let nodes = vec![action("a", &[]), action("a", &[])];
        assert!(Pipeline::create(pipeline_name(), project_id(), nodes).is_err());
    }

    #[test]
    fn rejects_duplicate_deps() {
        let nodes = vec![action("a", &[]), action("b", &["a", "a"])];
        assert!(Pipeline::create(pipeline_name(), project_id(), nodes).is_err());
    }

    #[test]
    fn update_nodes_preserves_on_failure() {
        let mut pipeline =
            Pipeline::create(pipeline_name(), project_id(), vec![action("a", &[])]).unwrap();

        let cyclic = vec![action("x", &["y"]), action("y", &["x"])];
        assert!(pipeline.update_nodes(cyclic).is_err());
        assert_eq!(pipeline.nodes().len(), 1);
    }

    #[test]
    fn update_nodes_replaces_the_whole_set() {
        let mut pipeline =
            Pipeline::create(pipeline_name(), project_id(), vec![action("a", &[])]).unwrap();

        let new_nodes = vec![action("x", &[]), action("y", &["x"])];
        pipeline.update_nodes(new_nodes).unwrap();

        assert_eq!(pipeline.nodes().len(), 2);
        assert!(pipeline.get_node(&node_id("x")).is_some());
        assert!(pipeline.get_node(&node_id("a")).is_none());
    }

    #[test]
    fn rejects_empty_action_command() {
        assert!(Step::exec("   ".into(), vec![]).is_err());
    }
}
