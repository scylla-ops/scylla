pub mod commands;
pub mod dispatch;
pub mod dispatch_port;
pub mod dispatch_use_case;
pub mod queries;
pub mod repository;
pub mod scheduler;

pub use commands::{CreateAgent, CreatedAgent, DeleteAgent, NewAgent};
pub use dispatch::{DispatchEnv, DispatchNode, JobDispatch};
pub use dispatch_port::AgentDispatch;
pub use dispatch_use_case::{DispatchOutcome, DispatchUseCases};
pub use queries::{AgentView, GetAgent, GetAgentStats, ListAgents};
pub use repository::{AgentRepository, AgentStats};
pub use scheduler::PendingJobScheduler;

use crate::application::{AppRepository, HashService};
use derive_more::Constructor;
use scylla_auth::authz::PolicyControl;
use std::sync::Arc;

/// The agent admin's stage runners, one block per action in `commands.rs` and `queries.rs`.
/// The agent's own stream and the dispatch run as the agent or the scheduler, outside the
/// pipeline.
#[derive(Constructor)]
pub struct AgentUseCases<A, W, H, PC>
where
    A: AppRepository,
    W: AgentRepository,
    H: HashService,
    PC: PolicyControl,
{
    pub(super) app_repo: Arc<A>,
    pub(super) agent_repo: Arc<W>,
    pub(super) hash_service: Arc<H>,
    pub(super) policy_control: Arc<PC>,
    pub(super) registry: Arc<dyn AgentDispatch>,
}

#[cfg(test)]
mod tests;
