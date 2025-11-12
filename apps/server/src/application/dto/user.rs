use crate::domain::entities::User;
use crate::domain::value_objects::{
    PaginationMetadata, PaginationParams, Password, UserGlobalRole, UserId, Username,
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct CreateUserRequestDto {
    pub username: Username,
    pub password: Password,
}

#[derive(Debug, Clone)]
pub struct UpdateUserRequestDto {
    pub user_id: UserId,
    pub username: Option<Username>,
}

#[derive(Debug, Clone)]
pub struct ChangeUserGlobalRoleRequestDto {
    pub user_id: UserId,
    pub new_role: UserGlobalRole,
    pub caller_id: Option<UserId>,
}

#[derive(Debug, Clone)]
pub struct ChangeUserGlobalRoleResponseDto {
    pub user_id: UserId,
    pub new_role: UserGlobalRole,
}

#[derive(Debug, Clone)]
pub struct GetUserRequestDto {
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct UserResponseDto {
    pub id: UserId,
    pub username: Username,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponseDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id().to_owned(),
            username: user.username().to_owned(),
            is_active: user.is_active(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        }
    }
}

impl From<&User> for UserResponseDto {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().to_owned(),
            username: user.username().to_owned(),
            is_active: user.is_active(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteUserRequestDto {
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct DeleteUserResponseDto {}

#[derive(Debug, Clone)]
pub struct ListUsersRequestDto {
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ListUsersResponseDto {
    pub users: Vec<UserResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}
