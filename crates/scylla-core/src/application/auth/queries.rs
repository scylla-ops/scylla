//! The session's reads. One block per query: the struct, its access, its output, what `Fetch`
//! reads.

use super::AuthUseCases;
use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use scylla_extension::{Access, Authorized, Describe, Fetch, Fetched, Query, Run};

/// No `Debug`: `token` is the session credential.
pub struct ValidateToken {
    pub token: String,
}

impl Describe for ValidateToken {
    fn access(&self) -> Access {
        Access::Public
    }
}

impl Query for ValidateToken {
    type Output = bool;
}

/// An expired session is deleted on the way, best-effort, as the interceptor does.
#[async_trait]
impl Run<Fetch<ValidateToken>> for AuthUseCases {
    async fn run(&self, input: Authorized<ValidateToken>) -> DomainResult<Fetched<ValidateToken>> {
        let token = input.command().token.as_str();
        let valid = if token.is_empty() {
            false
        } else {
            match self.session_repo.find_by_token(token).await {
                Ok(session) if session.is_expired() => {
                    let _ = self.session_repo.delete_by_token(token).await;
                    false
                }
                Ok(_) => true,
                Err(_) => false,
            }
        };
        Ok(input.fetched(valid))
    }
}
