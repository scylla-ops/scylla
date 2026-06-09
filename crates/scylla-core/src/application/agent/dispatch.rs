use crate::domain::value_objects::pipeline::Step;
use serde::{Deserialize, Serialize};

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
