use crate::api::grpc::user::dto::{NewUser, UpdateUser};
use crate::api::grpc::user::models::User;
use async_trait::async_trait;
pub mod controller;
mod dto;
pub mod models;
pub mod repo;
pub mod service;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, new_user: NewUser) -> anyhow::Result<User>;
    async fn get_user_by_uuid(&self, user_uuid: uuid::Uuid) -> anyhow::Result<Option<User>>;
    async fn list_users(&self, limit: i64, offset: i64) -> anyhow::Result<Vec<User>>;
    async fn update_user(
        &self,
        user_uuid: uuid::Uuid,
        changes: UpdateUser,
    ) -> anyhow::Result<Option<User>>;
    async fn deactivate_user(&self, user_uuid: uuid::Uuid) -> anyhow::Result<Option<User>>;
    async fn get_user_by_username(&self, username: String) -> anyhow::Result<Option<User>>;
}
