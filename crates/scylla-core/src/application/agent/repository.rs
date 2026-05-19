use crate::domain::entities::{Agent, AgentId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;

#[async_trait]
pub trait AgentRepository {
    async fn create(&self, agent: &Agent) -> DomainResult<Agent>;

    async fn find_by_id(&self, id: &AgentId) -> DomainResult<Agent>;

    async fn update(&self, agent: &Agent) -> DomainResult<Agent>;

    async fn delete(&self, id: &AgentId) -> DomainResult<()>;

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Agent>>;

    async fn exists(&self, id: &AgentId) -> DomainResult<bool>;
}
