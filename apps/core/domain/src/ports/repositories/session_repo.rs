use crate::entities::{Session, UserId};
use crate::errors::DomainResult;

/// Repository trait for Session management
pub trait SessionRepository: Send + Sync {
    /// Create a new session for a user
    fn create(&self, session: &Session) -> impl Future<Output = DomainResult<Session>> + Send;

    /// Find a session by its token
    fn find_by_token(&self, token: &str) -> impl Future<Output = DomainResult<Session>> + Send;

    /// Update a session
    fn update(&self, session: &Session) -> impl Future<Output = DomainResult<Session>> + Send;

    /// Delete a session by its token (logout)
    fn delete_by_token(&self, token: &str) -> impl Future<Output = DomainResult<()>> + Send;

    /// Delete all sessions for a user (logout everywhere)
    fn delete_all_for_user(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = DomainResult<u64>> + Send;

    /// Delete all expired sessions
    fn delete_expired(&self) -> impl Future<Output = DomainResult<u64>> + Send;

    /// List all active sessions for a user
    fn list_for_user(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = DomainResult<Vec<Session>>> + Send;
}
