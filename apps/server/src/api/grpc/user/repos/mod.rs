use crate::api::grpc::user::models::{InsertableUser, User, UserPatch};
use crate::api::grpc::utils::Id;
use async_trait::async_trait;

#[cfg(feature = "surreal")]
pub mod surreal;

const TABLE: &str = "users";

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create_user(new_user: InsertableUser) -> anyhow::Result<User>;
    async fn get_user_by_id(user_id: Id) -> anyhow::Result<Option<User>>;
    async fn get_user_by_username(username: String) -> anyhow::Result<Option<User>>;
    async fn list_users(limit: i64, offset: i64) -> anyhow::Result<Vec<User>>;
    async fn update_user(user_id: Id, changes: UserPatch) -> anyhow::Result<Option<User>>;
    async fn deactivate_user(user_id: Id) -> anyhow::Result<Option<User>>;
}
