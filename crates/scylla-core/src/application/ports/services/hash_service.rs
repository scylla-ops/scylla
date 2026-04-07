use crate::domain::errors::DomainResult;
use crate::domain::value_objects::user::{Password, PasswordHash};
use async_trait::async_trait;

#[async_trait]
pub trait HashService {
    async fn hash(&self, password: &Password) -> DomainResult<PasswordHash>;

    async fn verify(&self, password: &Password, hash: &PasswordHash) -> DomainResult<bool>;
}
