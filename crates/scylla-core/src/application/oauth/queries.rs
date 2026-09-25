//! The OAuth flow's reads. One block per query: the struct, its access, its output, what
//! `Fetch` reads.

use super::OAuthUseCases;
use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use scylla_extension::{Access, Authorized, Describe, Fetch, Fetched, Query, Run};

#[derive(Debug)]
pub struct GetAuthUrl {
    pub state: String,
}

impl Describe for GetAuthUrl {
    fn access(&self) -> Access {
        Access::Public
    }
}

impl Query for GetAuthUrl {
    type Output = String;
}

#[async_trait]
impl Run<Fetch<GetAuthUrl>> for OAuthUseCases {
    async fn run(&self, input: Authorized<GetAuthUrl>) -> DomainResult<Fetched<GetAuthUrl>> {
        let url = self.provider.authorize_url(&input.command().state)?;
        Ok(input.fetched(url))
    }
}
