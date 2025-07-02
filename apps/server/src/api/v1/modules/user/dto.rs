use crate::api::v1::models::users::User;
use diesel::Insertable;
use serde::{Deserialize, Serialize};
use validator::Validate;

// DB only
#[derive(Insertable, Deserialize, Validate)]
#[table_name = "crate::database::schema::users"]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
}

// Request DTO for creating a new user
#[derive(Deserialize, Validate)]
pub struct NewUserRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Username must be between 1 and 255 characters"
    ))]
    pub username: String,
    #[validate(length(
        min = 8,
        max = 255,
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

impl TryFrom<NewUserRequest> for NewUser {
    type Error = anyhow::Error;

    fn try_from(req: NewUserRequest) -> anyhow::Result<Self> {
        //todo: ne pas utiliser le coût par défaut en production
        let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
        Ok(Self {
            username: req.username,
            password_hash,
        })
    }
}
