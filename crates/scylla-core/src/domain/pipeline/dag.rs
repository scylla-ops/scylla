//! DAG bookkeeping for pipeline execution.
//!
//! Tracks in-degree, dependents, ready set, and pending set as nodes complete.
//!
//! This is the workspace's single implementation of Kahn's algorithm, and it
//! serves both sides on purpose. The control plane runs it once to reject a
//! pipeline containing a cycle ([`DagPlan::drains_completely`]); an agent runs
//! it incrementally to decide what to launch next. Two implementations could
//! disagree about what "ready" means, and the disagreement would only show up
//! at execution time, on a pipeline the server had already accepted.

use crate::domain::pipeline::PipelineNode;
use std::collections::{BTreeSet, HashMap};

/// Topological planner for a pipeline DAG.
///
/// Holds references into the caller-owned `Vec<PipelineNode>`. Uses `BTreeSet`
/// for the ready set so dispatch order is deterministic.
pub struct DagPlan<'a> {
    nodes: HashMap<&'a str, &'a PipelineNode>,
    in_degree: HashMap<&'a str, usize>,
    dependents: HashMap<&'a str, Vec<&'a str>>,
    ready: BTreeSet<&'a str>,
    pending: BTreeSet<&'a str>,
}

impl<'a> DagPlan<'a> {
    /// Build an initial plan from a slice of nodes.
    ///
    /// All roots (in-degree 0) are placed in the ready set.
    #[must_use]
    pub fn build(nodes: &'a [PipelineNode]) -> Self {
        let mut node_map: HashMap<&'a str, &'a PipelineNode> = HashMap::with_capacity(nodes.len());
        let mut in_degree: HashMap<&'a str, usize> = HashMap::with_capacity(nodes.len());
        let mut dependents: HashMap<&'a str, Vec<&'a str>> = HashMap::new();
        let mut pending: BTreeSet<&'a str> = BTreeSet::new();

        for node in nodes {
            let id = node.id().as_str();
            node_map.insert(id, node);
            in_degree.entry(id).or_insert(0);
            pending.insert(id);
        }

        for node in nodes {
            let id = node.id().as_str();
            for dep in node.deps() {
                *in_degree.entry(id).or_insert(0) += 1;
                dependents.entry(dep.as_str()).or_default().push(id);
            }
        }

        let ready: BTreeSet<&'a str> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&id, _)| id)
            .collect();

        Self {
            nodes: node_map,
            in_degree,
            dependents,
            ready,
            pending,
        }
    }

    /// Return all currently-ready nodes and clear the ready set.
    ///
    /// Caller is expected to dispatch each and later call `mark_completed`.
    pub fn drain_ready(&mut self) -> Vec<&'a str> {
        let batch: Vec<&'a str> = self.ready.iter().copied().collect();
        self.ready.clear();
        batch
    }

    /// Record that a node finished successfully: decrement its dependents'
    /// in-degree, push any newly-unblocked nodes into the ready set.
    pub fn mark_completed(&mut self, id: &str) {
        self.pending.remove(id);
        if let Some(deps) = self.dependents.get(id) {
            for &dep_id in deps {
                if let Some(deg) = self.in_degree.get_mut(dep_id) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 && self.pending.contains(dep_id) {
                        self.ready.insert(dep_id);
                    }
                }
            }
        }
    }

    /// Remove a node from the pending set without scheduling its dependents.
    ///
    /// Used for nodes marked Failed/Skipped during fail-fast propagation.
    pub fn mark_terminal(&mut self, id: &str) {
        self.pending.remove(id);
        self.ready.remove(id);
    }

    /// Iterate over all nodes that have not yet reached a terminal state.
    pub fn pending(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.pending.iter().copied()
    }

    /// True when every node has been marked terminal.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.pending.is_empty()
    }

    /// Look up the full node spec by id.
    #[must_use]
    pub fn lookup(&self, id: &str) -> &'a PipelineNode {
        self.nodes[id]
    }

    /// Drain the whole plan as if every node succeeded, and report whether every
    /// node was reached.
    ///
    /// A node that never becomes ready is one whose in-degree never falls to
    /// zero, which is exactly what a dependency cycle produces. This is the
    /// cycle check used by `Pipeline::create`.
    ///
    /// It assumes the structural checks have already run, and the two families
    /// behave differently, so neither check may be dropped as redundant:
    ///
    /// - A **dangling** dependency (pointing at no node) or a **self**
    ///   dependency leaves a node blocked forever, and is reported here as a
    ///   cycle. Running those checks first is what makes the error message
    ///   accurate.
    /// - A **duplicated** dependency or a **duplicated node id** drains
    ///   perfectly cleanly. A repeated dependency raises the in-degree twice and
    ///   also lands twice in the dependents list, so both decrements happen; a
    ///   repeated id is a single map key. This function returns `true` for both,
    ///   which makes `Pipeline::validate_nodes` their only rejection point.
    ///   Duplicated ids matter beyond validation: [`DagPlan::lookup`] would hand
    ///   an agent whichever node landed in the map last.
    #[must_use]
    pub fn drains_completely(mut self) -> bool {
        loop {
            let batch = self.drain_ready();
            if batch.is_empty() {
                return self.is_exhausted();
            }
            for id in batch {
                self.mark_completed(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::pipeline::{NodeId, Step};

    fn node(id: &str, deps: &[&str]) -> PipelineNode {
        PipelineNode::new(
            NodeId::new(id).unwrap(),
            deps.iter().map(|d| NodeId::new(*d).unwrap()).collect(),
            Step::exec("echo".into(), vec![]).unwrap(),
            None,
            vec![],
        )
    }

    #[test]
    fn initial_ready_is_roots() {
        let nodes = vec![node("a", &[]), node("b", &["a"]), node("c", &["a"])];
        let mut plan = DagPlan::build(&nodes);

        let batch = plan.drain_ready();
        assert_eq!(batch, vec!["a"]);
        assert!(!plan.is_exhausted());
    }

    #[test]
    fn mark_completed_unblocks_dependents() {
        let nodes = vec![node("a", &[]), node("b", &["a"]), node("c", &["a"])];
        let mut plan = DagPlan::build(&nodes);

        let _ = plan.drain_ready();
        plan.mark_completed("a");

        let mut batch = plan.drain_ready();
        batch.sort_unstable();
        assert_eq!(batch, vec!["b", "c"]);
    }

    #[test]
    fn diamond_order() {
        // a -> b, a -> c, d depends on b+c
        let nodes = vec![
            node("a", &[]),
            node("b", &["a"]),
            node("c", &["a"]),
            node("d", &["b", "c"]),
        ];
        let mut plan = DagPlan::build(&nodes);

        assert_eq!(plan.drain_ready(), vec!["a"]);
        plan.mark_completed("a");

        let mut batch = plan.drain_ready();
        batch.sort_unstable();
        assert_eq!(batch, vec!["b", "c"]);
        plan.mark_completed("b");
        assert!(plan.drain_ready().is_empty(), "d blocked by c");

        plan.mark_completed("c");
        assert_eq!(plan.drain_ready(), vec!["d"]);
        plan.mark_completed("d");

        assert!(plan.is_exhausted());
    }

    #[test]
    fn drains_completely_accepts_an_acyclic_graph() {
        let nodes = vec![
            node("a", &[]),
            node("b", &["a"]),
            node("c", &["a"]),
            node("d", &["b", "c"]),
        ];
        assert!(DagPlan::build(&nodes).drains_completely());
    }

    #[test]
    fn drains_completely_rejects_a_cycle() {
        // a -> b -> c -> a: no node ever reaches in-degree 0.
        let nodes = vec![node("a", &["c"]), node("b", &["a"]), node("c", &["b"])];
        assert!(!DagPlan::build(&nodes).drains_completely());
    }

    /// The counterpart of the two tests above, and the reason
    /// `Pipeline::validate_nodes` cannot drop its structural checks: these two
    /// malformed graphs drain perfectly cleanly, so cycle detection waves them
    /// through. A repeated dependency raises the in-degree twice and also lands
    /// twice in the dependents list, so both decrements happen. A repeated node
    /// id is a single map key.
    #[test]
    fn drains_completely_does_not_catch_structural_problems() {
        let duplicated_dep = vec![node("a", &[]), node("b", &["a", "a"])];
        assert!(DagPlan::build(&duplicated_dep).drains_completely());

        let duplicated_id = vec![node("a", &[]), node("a", &[])];
        assert!(DagPlan::build(&duplicated_id).drains_completely());
    }

    /// A dangling or self dependency, on the other hand, does look exactly like
    /// a cycle here. That is why the structural checks run first: they own the
    /// accurate error message.
    #[test]
    fn drains_completely_reports_unreachable_dependencies_as_a_cycle() {
        let dangling = vec![node("a", &[]), node("b", &["ghost"])];
        assert!(!DagPlan::build(&dangling).drains_completely());

        let self_dep = vec![node("a", &["a"])];
        assert!(!DagPlan::build(&self_dep).drains_completely());
    }

    #[test]
    fn drains_completely_rejects_a_cycle_hanging_off_a_valid_root() {
        // `root` runs fine, but x and y deadlock on each other.
        let nodes = vec![
            node("root", &[]),
            node("x", &["root", "y"]),
            node("y", &["x"]),
        ];
        assert!(!DagPlan::build(&nodes).drains_completely());
    }

    #[test]
    fn pending_excludes_marked_terminal() {
        let nodes = vec![node("a", &[]), node("b", &["a"])];
        let mut plan = DagPlan::build(&nodes);

        plan.mark_terminal("a");
        let pending: Vec<&str> = plan.pending().collect();
        assert_eq!(pending, vec!["b"]);
    }

    #[test]
    fn mark_terminal_on_failure_leaves_dependents_pending() {
        let nodes = vec![node("a", &[]), node("b", &["a"])];
        let mut plan = DagPlan::build(&nodes);

        let _ = plan.drain_ready();
        plan.mark_terminal("a");

        // b must stay pending (not ready, not completed)
        assert!(plan.drain_ready().is_empty());
        let pending: Vec<&str> = plan.pending().collect();
        assert_eq!(pending, vec!["b"]);
    }
}
