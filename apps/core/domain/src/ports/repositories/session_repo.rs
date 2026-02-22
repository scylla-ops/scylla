use crate::entities::{Session, UserId};
use crate::errors::DomainResult;
use async_trait::async_trait;

/// Repository trait for Session management
#[async_trait]
pub trait SessionRepository {
    /// Create a new session for a user
    async fn create(&self, session: &Session) -> DomainResult<Session>;

    /// Find a session by its token
    async fn find_by_token(&self, token: &str) -> DomainResult<Session>;

    /// Update a session
    async fn update(&self, session: &Session) -> DomainResult<Session>;

    /// Delete a session by its token (logout)
    async fn delete_by_token(&self, token: &str) -> DomainResult<()>;

    /// Delete all expired sessions
    async fn delete_expired(&self) -> DomainResult<u64>;

    /// List all active sessions for a user
    async fn list_for_user(&self, user_id: &UserId) -> DomainResult<Vec<Session>>;
}
