use crate::api::v1::modules::user::dto::{NewUserRequest, UserResponse};
use crate::api::v1::modules::user::repository::UserRepository;
use anyhow::Result;
use uuid::Uuid;

// Command service for handling command-related business logic
pub struct UserService {
    repository: UserRepository,
}

impl UserService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    // Create a new user
    pub async fn create_user(&self, req: NewUserRequest) -> Result<()> {
        let res = self.repository.create_user(req.try_into()?).await?;
        Ok(())
    }

    // Get user by ID
    pub async fn get_user_by_id(&self, user_uuid: Uuid) -> Result<Option<UserResponse>> {
        let user = self.repository.get_user_by_uuid(user_uuid).await?;
        Ok(user.map(UserResponse::from))
    }
}
