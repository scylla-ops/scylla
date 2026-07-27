use crate::application::pipeline::repository::PipelineRepository;
use crate::application::secret::SecretResolver;
use crate::domain::entities::Job;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::pipeline::Step;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Everything an agent needs to execute a pipeline job, handed to a connected
/// agent through the [`AgentDispatch`](crate::application::AgentDispatch) port.
/// This is an application/transport payload (the port's data contract), not a
/// domain value object. Its nodes are **resolved**: every env var carries a
/// concrete value (secret references already decrypted control-plane-side), with
/// `masked` marking values that came from a secret so the agent can scrub them
/// from logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDispatch {
    pub job_id: String,
    pub pipeline_id: String,
    pub nodes: Vec<DispatchNode>,
}

/// A pipeline node prepared for dispatch: identity + deps + the resolved step
/// and environment. Mirrors the domain `PipelineNode` but with env resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchNode {
    pub id: String,
    pub deps: Vec<String>,
    pub working_dir: Option<String>,
    pub step: Step,
    pub env: Vec<DispatchEnv>,
}

/// A fully-resolved environment variable for dispatch. `masked` is true when the
/// value originated from a secret (the agent redacts it from log output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchEnv {
    pub key: String,
    pub value: String,
    pub masked: bool,
}

/// THE single way to turn a persisted [`Job`] into a [`JobDispatch`]: load its
/// pipeline, resolve secret-ref env (decrypt), then overlay the job's stored
/// trigger inputs as literal env. Both the immediate run path
/// ([`PipelineUseCases::run_with_inputs`](crate::application::PipelineUseCases))
/// and the pending-job retry path
/// ([`PendingJobScheduler`](crate::application::PendingJobScheduler)) go through
/// here, so a job dispatches identically however it is placed — there is no
/// second, diverging assembly. Secrets are re-resolved here (never persisted
/// decrypted); only the literal inputs are stored on the job.
pub async fn assemble_dispatch<P>(
    pipeline_repo: &P,
    secret_resolver: &dyn SecretResolver,
    job: &Job,
) -> DomainResult<JobDispatch>
where
    P: PipelineRepository + ?Sized,
{
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

/// Overlay a job's literal `inputs` onto each dispatch node as unmasked env.
/// Applied after secret resolution; a node's own env wins on a key collision, so
/// a trigger can add context (e.g. `GIT_COMMIT`) but never override or shadow
/// what the pipeline defined. Inputs are plain literals and can never reference a
/// secret.
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
