pub mod commands;
pub mod hash_service;
pub mod queries;
pub mod session_repository;

pub use commands::{Login, RevokeToken};
pub use hash_service::HashService;
pub use queries::ValidateToken;
pub use session_repository::SessionRepository;

use crate::application::UserRepository;
use crate::domain::ids::UserId;
use crate::domain::session::Session;
use chrono::Duration;
use derive_more::Constructor;
use std::sync::Arc;
use uuid::Uuid;

const SESSION_TTL_HOURS: i64 = 24;

/// A random opaque token, valid for one day. Every sign-in path stages one.
#[must_use]
pub fn new_session(user_id: UserId) -> Session {
    Session::create(
        user_id,
        Uuid::new_v4().to_string(),
        Duration::hours(SESSION_TTL_HOURS),
    )
}

/// The session's stage runners, one block per action in `commands.rs` and `queries.rs`. Every
/// action is `Public`: the caller holds no session yet, or holds the token it sends.
#[derive(Constructor)]
pub struct AuthUseCases {
    pub(super) user_repo: Arc<dyn UserRepository>,
    pub(super) session_repo: Arc<dyn SessionRepository>,
    pub(super) hash_service: Arc<dyn HashService>,
}

#[cfg(test)]
mod tests;
