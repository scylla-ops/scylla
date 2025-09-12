use crate::api::grpc::user::models::User;
use diesel::{AsChangeset, Insertable};
use protocol::services::{CreateUserRequest, UserResponse};
use validator::Validate;

pub const USERNAME_MIN_LENGTH: u64 = 1;
pub const USERNAME_MAX_LENGTH: u64 = 255;
pub const PASSWORD_MIN_LENGTH: u64 = 8;
pub const PASSWORD_MAX_LENGTH: u64 = 255;

#[derive(Debug, Clone, Validate)]
pub struct UserFields {
    #[validate(length(
        min = USERNAME_MIN_LENGTH,
        max = USERNAME_MAX_LENGTH,
        message = "must be between 1 and 255 characters"
    ))]
    pub username: Option<String>,

    #[validate(length(
        min = PASSWORD_MIN_LENGTH,
        max = PASSWORD_MAX_LENGTH,
        message = "must be between 8 and 255 characters"
    ))]
    pub password: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Validate)]
pub struct NewUserRequest {
    #[validate(nested)]
    pub fields: UserFields,
}

#[derive(Debug, Validate)]
pub struct UpdateUserRequest {
    #[validate(nested)]
    pub fields: UserFields,
}

// DB only
#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::users)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::database::schema::users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<CreateUserRequest> for NewUserRequest {
    fn from(value: CreateUserRequest) -> Self {
        NewUserRequest {
            fields: UserFields {
                username: Option::from(value.username),
                password: Option::from(value.password),
                is_active: None,
            },
        }
    }
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        UserResponse {
            user_uuid: value.id.to_string(),
            username: value.username,
            password_hash: value.password_hash,
            is_active: value.is_active,
            created_at: value.created_at.to_string(),
            updated_at: value.updated_at.to_string(),
        }
    }
}

impl From<protocol::services::UpdateUserRequest> for UpdateUserRequest {
    fn from(value: protocol::services::UpdateUserRequest) -> Self {
        Self {
            fields: UserFields {
                username: value.username,
                password: value.password,
                is_active: None,
            },
        }
    }
}
