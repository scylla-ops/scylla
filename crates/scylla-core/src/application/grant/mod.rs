pub mod commands;
pub mod queries;

pub use commands::{CreateGrant, RevokeAllAccess, RevokeGrant};
pub use queries::{ListGrantableRoles, ListGrants};

use crate::application::agent::dispatch_port::AgentDispatch;
use derive_more::Constructor;
use scylla_auth::authz::{AuthzEntityProvider, GrantRepository, PolicyControl, RoleRepository};
use std::sync::Arc;

/// The grant aggregate's stage runners, one block per action in `commands.rs` and `queries.rs`.
/// It has no method of its own; `Actions::run` drives it.
#[derive(Constructor)]
pub struct GrantUseCases {
    pub(super) grant_repo: Arc<dyn GrantRepository>,
    pub(super) role_repo: Arc<dyn RoleRepository>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
    pub(super) agent_registry: Arc<dyn AgentDispatch>,
    pub(super) entity_provider: Arc<dyn AuthzEntityProvider>,
}

#[cfg(test)]
mod tests;
