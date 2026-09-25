use crate::application::pipeline::repository::PipelineRepository;
use crate::application::secret::SecretResolver;
use crate::domain::errors::DomainResult;
use crate::domain::job::Job;
use crate::domain::pipeline::Step;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDispatch {
    pub job_id: String,
    pub pipeline_id: String,
    pub nodes: Vec<DispatchNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchNode {
    pub id: String,
    pub deps: Vec<String>,
    pub working_dir: Option<String>,
    pub step: Step,
    pub env: Vec<DispatchEnv>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchEnv {
    pub key: String,
    pub value: String,
    pub masked: bool,
}

/// Both the immediate run and the pending retry go through here so a job dispatches identically.
pub async fn assemble_dispatch(
    pipeline_repo: &dyn PipelineRepository,
    secret_resolver: &dyn SecretResolver,
    job: &Job,
) -> DomainResult<JobDispatch> {
    let pipeline = pipeline_repo.find_by_id(job.pipeline_id()).await?;
    let nodes = secret_resolver
        .resolve(pipeline.project_id(), pipeline.nodes())
        .await?;
    let nodes = apply_inputs(nodes, job.inputs());
    Ok(JobDispatch {
        job_id: job.id().to_string(),
        pipeline_id: pipeline.id().to_string(),
        nodes,
    })
}

/// A node's own env wins on a key collision: a trigger adds context, never overrides.
fn apply_inputs(mut nodes: Vec<DispatchNode>, inputs: &[(String, String)]) -> Vec<DispatchNode> {
    if inputs.is_empty() {
        return nodes;
    }
    for node in &mut nodes {
        let existing: HashSet<String> = node.env.iter().map(|e| e.key.clone()).collect();
        for (key, value) in inputs {
            if !existing.contains(key) {
                node.env.push(DispatchEnv {
                    key: key.clone(),
                    value: value.clone(),
                    masked: false,
                });
            }
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(env: &[(&str, &str)]) -> DispatchNode {
        DispatchNode {
            id: "n".to_string(),
            deps: vec![],
            working_dir: None,
            step: Step::exec("echo".to_string(), vec![]).unwrap(),
            env: env
                .iter()
                .map(|(k, v)| DispatchEnv {
                    key: (*k).to_string(),
                    value: (*v).to_string(),
                    masked: false,
                })
                .collect(),
        }
    }

    fn env_of(node: &DispatchNode) -> Vec<(String, String)> {
        node.env
            .iter()
            .map(|e| (e.key.clone(), e.value.clone()))
            .collect()
    }

    #[test]
    fn empty_inputs_leave_nodes_untouched() {
        let nodes = vec![node(&[("A", "1")])];
        let out = apply_inputs(nodes, &[]);
        assert_eq!(env_of(&out[0]), vec![("A".into(), "1".into())]);
    }

    #[test]
    fn inputs_are_appended_as_literals() {
        let out = apply_inputs(vec![node(&[])], &[("GIT_COMMIT".into(), "abc".into())]);
        assert_eq!(env_of(&out[0]), vec![("GIT_COMMIT".into(), "abc".into())]);
        assert!(!out[0].env[0].masked);
    }

    #[test]
    fn node_env_wins_on_collision() {
        let out = apply_inputs(
            vec![node(&[("GIT_COMMIT", "from-node")])],
            &[("GIT_COMMIT".into(), "from-input".into())],
        );
        assert_eq!(out[0].env.len(), 1);
        assert_eq!(out[0].env[0].value, "from-node");
    }
}
