use crate::api::v1::models::users::User;
use diesel::{AsChangeset, Insertable};
use serde::{Deserialize, Serialize};
use validator::Validate;

const USERNAME_MIN_LENGTH: u64 = 1;
const USERNAME_MAX_LENGTH: u64 = 255;
const PASSWORD_MIN_LENGTH: u64 = 8;
const PASSWORD_MAX_LENGTH: u64 = 255;

// DB only
#[derive(Insertable, Deserialize, Validate)]
#[diesel(table_name = crate::database::schema::users)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
}

// Request DTO for creating a new user
#[derive(Deserialize, Validate)]
pub struct NewUserRequest {
    #[validate(length(
        min = USERNAME_MIN_LENGTH,
        max = USERNAME_MAX_LENGTH,
        message = "Username must be between 1 and 255 characters"
    ))]
    pub username: String,
    #[validate(length(
        min = PASSWORD_MIN_LENGTH,
        max = PASSWORD_MAX_LENGTH,
        message = "Password must be between 8 and 255 characters"
    ))]
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub uuid: uuid::Uuid,
    pub username: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            uuid: user.id,
            username: user.username,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

fn hash_password(password: &str) -> anyhow::Result<String> {
    //todo: ne pas utiliser le coût par défaut en production
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))
}

impl TryFrom<NewUserRequest> for NewUser {
    type Error = anyhow::Error;

    fn try_from(req: NewUserRequest) -> anyhow::Result<Self> {
        let password_hash = hash_password(&req.password)?;
        Ok(Self {
            username: req.username,
            password_hash,
        })
    }
}

#[derive(Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(
        min = USERNAME_MIN_LENGTH,
        max = USERNAME_MAX_LENGTH,
        message = "Username must be between 1 and 255 characters"
    ))]
    pub username: Option<String>,
    pub is_active: Option<bool>,
    #[validate(length(
        min = PASSWORD_MIN_LENGTH,
        max = PASSWORD_MAX_LENGTH,
        message = "Password must be between 8 and 255 characters"
    ))]
    pub password: Option<String>,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::database::schema::users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub is_active: Option<bool>,
    pub password_hash: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
}

impl Default for UpdateUser {
    fn default() -> Self {
        Self {
            username: None,
            is_active: None,
            password_hash: None,
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl TryFrom<UpdateUserRequest> for UpdateUser {
    type Error = anyhow::Error;

    fn try_from(req: UpdateUserRequest) -> anyhow::Result<Self> {
        let password_hash = if let Some(password) = req.password {
            Some(hash_password(&password)?)
        } else {
            None
        };

        Ok(Self {
            username: req.username,
            is_active: req.is_active,
            password_hash,
            updated_at: chrono::Utc::now().naive_utc(),
        })
    }
}
