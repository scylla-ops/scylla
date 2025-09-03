use crate::api::v1::common::base::DieselRepository;
use crate::api::v1::modules::user::dto::{
    NewUserRequest, UpdateUserRequest, UserDtoError, UserResponse,
};
use crate::api::v1::modules::user::repository::UserRepositoryTrait;
use crate::handle_diesel_result;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;
use thiserror::Error;
use uuid::Uuid;

pub struct UserService<R: Repository + UserRepositoryTrait> {
    repository: R,
}

#[derive(Error, Debug)]
pub enum UserServiceError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error(transparent)]
    ValidationError(#[from] UserDtoError),

    #[error("{0}")]
    RepositoryError(#[from] anyhow::Error),
}

type ServiceResult<T> = Result<T, UserServiceError>;

pub trait UserServiceTrait<R: Repository + UserRepositoryTrait> {
    fn new(repository: R) -> Self;
    async fn create_user(&self, req: NewUserRequest) -> ServiceResult<usize>;
    async fn get_user_by_id(&self, user_uuid: Uuid) -> ServiceResult<Option<UserResponse>>;
    async fn get_all_users(&self) -> ServiceResult<Vec<UserResponse>>;
    async fn update_user_by_id(&self, user_uuid: Uuid, req: UpdateUserRequest)
    -> ServiceResult<()>;
    async fn deactivate_user_by_id(&self, user_uuid: Uuid) -> ServiceResult<()>;
}

impl<R: Repository + UserRepositoryTrait> UserServiceTrait<R> for UserService<R> {
    fn new(repository: R) -> Self {
        Self { repository }
    }

    // Create a new user
    async fn create_user(&self, req: NewUserRequest) -> ServiceResult<usize> {
        let res = self.repository.create_user(req.try_into()?).await;
        handle_diesel_result!(res,
            {DatabaseError(DatabaseErrorKind::UniqueViolation, _) => UserServiceError::UserAlreadyExists}
        )
    }

    // Get user by ID
    async fn get_user_by_id(&self, user_uuid: Uuid) -> ServiceResult<Option<UserResponse>> {
        Ok(self
            .repository
            .get_user_by_uuid(user_uuid)
            .await?
            .map(UserResponse::from))
    }

    // Get all users
    async fn get_all_users(&self) -> ServiceResult<Vec<UserResponse>> {
        let users = self.repository.get_all_users().await?;
        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    // Update user by ID
    async fn update_user_by_id(
        &self,
        user_uuid: Uuid,
        req: UpdateUserRequest,
    ) -> ServiceResult<()> {
        self.repository
            .update_user_by_uuid(user_uuid, req.try_into()?)
            .await?;
        Ok(())
    }

    // Deactivate user by ID
    async fn deactivate_user_by_id(&self, user_uuid: Uuid) -> ServiceResult<()> {
        self.repository.deactivate_user_by_uuid(user_uuid).await?;
        Ok(())
    }
}
