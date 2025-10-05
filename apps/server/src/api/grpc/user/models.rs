use crate::api::grpc::user::username::ScyllaUsername;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

// Domain
#[derive(Debug, Clone)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct UpdateUserInput {
    pub username: Option<String>,
    pub password: Option<String>,
    pub is_active: Option<bool>,
}

// DB
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub username: ScyllaUsername,
    pub password_hash: String,
    pub is_active: bool,
    #[serde(skip_serializing)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct UserPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<ScyllaUsername>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

#[derive(Serialize)]
pub struct InsertableUser {
    pub username: ScyllaUsername,
    pub password_hash: String,
}
