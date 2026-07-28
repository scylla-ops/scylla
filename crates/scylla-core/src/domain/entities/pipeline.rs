use crate::domain::clock;
use crate::domain::entities::{PipelineId, ProjectId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::pipeline::{EnvVar, NodeId, PipelineName, Step, WorkingDir};
use chrono::{DateTime, Utc};
use crate::domain::dag::DagPlan;
use std::collections::HashSet;

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

        // Cycle detection runs last, on purpose: DagPlan reports a node that can
        // never become ready, and a dangling or duplicated dependency produces
        // that same symptom. The checks above rule both out first, so anything
        // still stuck here really is a cycle.
        if !DagPlan::build(nodes).drains_completely() {
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

    /// Golden test for the `pipelines.nodes` JSONB column.
    ///
    /// The blob below is the on-disk shape. Renaming a field, changing a serde
    /// tag, or dropping one of the two `#[serde(default)]` attributes would make
    /// every pipeline already stored in a deployed database unreadable, and the
    /// failure would surface as an opaque decode error at query time rather than
    /// at compile time. This test is the only thing that turns that into a build
    /// failure, so update it only together with a deliberate migration.
    #[test]
    fn pipeline_nodes_jsonb_shape_is_stable() {
        const STORED: &str = r#"[
            {"id":"build","deps":[],"working_dir":"crates/api",
             "env":[{"key":"RUST_LOG","source":{"literal":"debug"}}],
             "step":{"kind":"exec","command":"cargo","args":["build"]}},
            {"id":"test","deps":["build"],"working_dir":null,"env":[],
             "step":{"kind":"script","script":"cargo test\n","shell":"bash"}}
        ]"#;

        let nodes: Vec<PipelineNode> = serde_json::from_str(STORED).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id().as_str(), "build");
        assert_eq!(nodes[0].working_dir().unwrap().as_str(), "crates/api");
        assert_eq!(nodes[0].env().len(), 1);
        assert!(nodes[1].working_dir().is_none());
        assert_eq!(nodes[1].deps(), &[node_id("build")]);

        let round_tripped: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&nodes).unwrap()).unwrap();
        let original: serde_json::Value = serde_json::from_str(STORED).unwrap();
        assert_eq!(round_tripped, original, "serialized shape drifted");
    }

    /// Rows written before `working_dir` and `env` existed omit both keys.
    /// The two `#[serde(default)]` attributes are what keeps them readable.
    #[test]
    fn pipeline_nodes_jsonb_tolerates_missing_optional_keys() {
        const LEGACY: &str =
            r#"[{"id":"a","deps":[],"step":{"kind":"script","script":"ls","shell":"sh"}}]"#;

        let nodes: Vec<PipelineNode> = serde_json::from_str(LEGACY).unwrap();
        assert!(nodes[0].working_dir().is_none());
        assert!(nodes[0].env().is_empty());
    }
}
