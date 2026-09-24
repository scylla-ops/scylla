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
pub struct UserUseCases {
    pub(super) user_repo: Arc<dyn UserRepository>,
    pub(super) hash_service: Arc<dyn HashService>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
}

#[cfg(test)]
mod tests;
