use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::DomainResult;
use crate::domain::ids::UserId;
use crate::domain::user::User;
use crate::domain::user::{Email, Username};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> DomainResult<User>;

    async fn find_by_id(&self, id: &UserId) -> DomainResult<User>;

    async fn find_by_ids(&self, ids: &[UserId]) -> DomainResult<Vec<User>>;

    async fn find_by_username(&self, username: &Username) -> DomainResult<User>;

    async fn find_by_email(&self, email: &Email) -> DomainResult<User>;

    async fn update(&self, user: &User) -> DomainResult<User>;

    async fn delete(&self, id: &UserId) -> DomainResult<()>;

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>>;

    async fn username_exists(&self, username: &Username) -> DomainResult<bool>;
}

/// Resolves a page of ids to users in the page's order; an id with no user is dropped.
pub async fn users_in_order(
    repo: &dyn UserRepository,
    page: PaginatedResult<UserId>,
) -> DomainResult<PaginatedResult<User>> {
    let (user_ids, metadata) = page.into_parts();
    let mut by_id: HashMap<String, User> = repo
        .find_by_ids(&user_ids)
        .await?
        .into_iter()
        .map(|u| (u.id().as_str().to_owned(), u))
        .collect();
    let users = user_ids
        .iter()
        .filter_map(|id| by_id.remove(id.as_str()))
        .collect();
    Ok(PaginatedResult::from_parts(users, metadata))
}
