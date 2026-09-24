//! The user's reads. One block per query, in the order it runs: the struct, its
//! permission, its output type, what `Fetch` reads.

use super::UserUseCases;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::DomainResult;
use crate::domain::ids::UserId;
use crate::domain::permission::Permission;
use crate::domain::user::{User, Username};
use async_trait::async_trait;
use scylla_extension::{Authorized, Describe, Fetch, Fetched, Query, Run};

#[derive(Debug)]
pub struct GetUser {
    pub id: UserId,
}

impl Describe for GetUser {
    fn permission(&self) -> Permission {
        Permission::ReadUser(self.id.clone())
    }
}

impl Query for GetUser {
    type Output = User;
}

#[async_trait]
impl Run<Fetch<GetUser>> for UserUseCases {
    async fn run(&self, input: Authorized<GetUser>) -> DomainResult<Fetched<GetUser>> {
        let user = self.user_repo.find_by_id(&input.command().id).await?;
        Ok(input.fetched(user))
    }
}

/// No RPC sends it; the bootstrap reads back an admin that already exists.
#[derive(Debug)]
pub struct GetUserByUsername {
    pub username: Username,
}

impl Describe for GetUserByUsername {
    fn permission(&self) -> Permission {
        Permission::ListUsers
    }
}

impl Query for GetUserByUsername {
    type Output = User;
}

#[async_trait]
impl Run<Fetch<GetUserByUsername>> for UserUseCases {
    async fn run(
        &self,
        input: Authorized<GetUserByUsername>,
    ) -> DomainResult<Fetched<GetUserByUsername>> {
        let user = self
            .user_repo
            .find_by_username(&input.command().username)
            .await?;
        Ok(input.fetched(user))
    }
}

#[derive(Debug)]
pub struct ListUsers {
    pub pagination: Option<PaginationParams>,
}

impl Describe for ListUsers {
    fn permission(&self) -> Permission {
        Permission::ListUsers
    }
}

impl Query for ListUsers {
    type Output = PaginatedResult<User>;
}

#[async_trait]
impl Run<Fetch<ListUsers>> for UserUseCases {
    async fn run(&self, input: Authorized<ListUsers>) -> DomainResult<Fetched<ListUsers>> {
        let page = self
            .user_repo
            .list_all(input.command().pagination.as_ref())
            .await?;
        Ok(input.fetched(page))
    }
}
