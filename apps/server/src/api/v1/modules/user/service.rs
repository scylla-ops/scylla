use crate::api::v1::common::base::Repository;
use crate::api::v1::modules::user::dto::{NewUserRequest, UpdateUserRequest, UserResponse};
use crate::api::v1::modules::user::repository::UserRepositoryTrait;
use anyhow::Result;
use uuid::Uuid;

// Command service for handling command-related business logic
pub struct UserService<R: Repository + UserRepositoryTrait> {
    repository: R,
}

pub trait UserServiceTrait<R: Repository + UserRepositoryTrait> {
    fn new(repository: R) -> Self;
    async fn create_user(&self, req: NewUserRequest) -> Result<usize>;
    async fn get_user_by_id(&self, user_uuid: Uuid) -> Result<Option<UserResponse>>;
    async fn get_all_users(&self) -> Result<Vec<UserResponse>>;
    async fn update_user_by_id(&self, user_uuid: Uuid, req: UpdateUserRequest) -> Result<()>;
    async fn deactivate_user_by_id(&self, user_uuid: Uuid) -> Result<()>;
}

impl<R: Repository + UserRepositoryTrait> UserServiceTrait<R> for UserService<R> {
    fn new(repository: R) -> Self {
        Self { repository }
    }

    // Create a new user
    async fn create_user(&self, req: NewUserRequest) -> Result<usize> {
        self.repository.create_user(req.try_into()?).await
    }

    // Get user by ID
    async fn get_user_by_id(&self, user_uuid: Uuid) -> Result<Option<UserResponse>> {
        Ok(self
            .repository
            .get_user_by_uuid(user_uuid)
            .await?
            .map(UserResponse::from))
    }

    // Get all users
    async fn get_all_users(&self) -> Result<Vec<UserResponse>> {
        let users = self.repository.get_all_users().await?;
        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    // Update user by ID
    async fn update_user_by_id(&self, user_uuid: Uuid, req: UpdateUserRequest) -> Result<()> {
        match self
            .repository
            .update_user_by_uuid(user_uuid, req.try_into()?)
            .await?
        {
            0 => Err(anyhow::anyhow!("User not found")),
            _ => Ok(()),
        }
    }

    // Deactivate user by ID
    async fn deactivate_user_by_id(&self, user_uuid: Uuid) -> Result<()> {
        match self.repository.deactivate_user_by_uuid(user_uuid).await? {
            0 => Err(anyhow::anyhow!("User not found")),
            _ => Ok(()),
        }
    }
}
