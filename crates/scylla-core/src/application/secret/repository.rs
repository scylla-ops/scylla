use crate::domain::entities::{ProjectId, Secret, SecretId};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Persistence for project secrets. Stores ciphertext only; plaintext is never
/// persisted and never returned.
#[async_trait]
pub trait SecretRepository: Send + Sync {
    /// Insert a new secret. Fails with a conflict if `(project_id, name)` exists.
    async fn create(&self, secret: &Secret) -> DomainResult<()>;
    /// One secret by id (with ciphertext), or not-found.
    async fn find_by_id(&self, id: &SecretId) -> DomainResult<Secret>;
    /// Every secret in a project (with ciphertext — callers decide what to expose).
    async fn list_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Secret>>;
    /// Delete a secret by id.
    async fn delete(&self, id: &SecretId) -> DomainResult<()>;
}
