pub mod commands;
pub mod queries;
pub mod repository;

pub use commands::{CreateUser, DeleteUser, UpdateUser};
pub use queries::{GetUser, GetUserByUsername, ListUsers};
pub use repository::UserRepository;

use crate::application::HashService;
use derive_more::Constructor;
use scylla_auth::authz::PolicyControl;
use std::sync::Arc;

/// The user aggregate's stage runners, one block per action in `commands.rs` and
/// `queries.rs`. It has no method of its own; `Actions::run` drives it.
#[derive(Constructor)]
pub struct UserUseCases<U: UserRepository, H: HashService, PC: PolicyControl> {
    pub(super) user_repo: Arc<U>,
    pub(super) hash_service: Arc<H>,
    pub(super) policy_control: Arc<PC>,
}

#[cfg(test)]
mod tests;
