pub mod commands;
pub mod dispatch;
pub mod dispatch_port;
pub mod dispatch_use_case;
pub mod queries;
pub mod repository;
pub mod scheduler;

pub use commands::{CreateAgent, CreatedAgent, DeleteAgent, NewAgent, RecordAgentHost, TouchAgent};
pub use dispatch::{DispatchEnv, DispatchNode, JobDispatch};
pub use dispatch_port::AgentDispatch;
pub use dispatch_use_case::{DispatchOutcome, DispatchPendingJobs, DispatchUseCases};
pub use queries::{AgentView, GetAgent, GetAgentStats, ListAgents};
pub use repository::{AgentRepository, AgentStats};
pub use scheduler::PendingJobScheduler;

use crate::application::{AppRepository, HashService};
use derive_more::Constructor;
use scylla_auth::authz::PolicyControl;
use std::sync::Arc;

/// The agent's stage runners, one block per action in `commands.rs` and `queries.rs`: the
/// admin actions, and the heartbeat and host report of the agent's own stream.
#[derive(Constructor)]
pub struct AgentUseCases {
    pub(super) app_repo: Arc<dyn AppRepository>,
    pub(super) agent_repo: Arc<dyn AgentRepository>,
    pub(super) hash_service: Arc<dyn HashService>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
    pub(super) registry: Arc<dyn AgentDispatch>,
}

#[cfg(test)]
mod tests;
