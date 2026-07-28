use crate::domain::errors::DomainResult;
use crate::domain::ids::UserId;
use crate::domain::session::Session;
use async_trait::async_trait;

#[async_trait]
pub trait SessionRepository {
    async fn create(&self, session: &Session) -> DomainResult<Session>;

    async fn find_by_token(&self, token: &str) -> DomainResult<Session>;

    async fn update(&self, session: &Session) -> DomainResult<Session>;

    async fn delete_by_token(&self, token: &str) -> DomainResult<()>;

    async fn delete_expired(&self) -> DomainResult<u64>;

    async fn list_for_user(&self, user_id: &UserId) -> DomainResult<Vec<Session>>;
}
