use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use surrealdb::sql::Datetime;

/// User database record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub username: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Serialize)]
pub struct UserInsert {
    pub username: String,
    pub password_hash: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct UserUpdate {
    pub username: String,
    pub is_active: bool,
}

/// Organization database record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationRecord {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Serialize)]
pub struct OrganizationInsert {
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct OrganizationUpdate {
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

/// Project database record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing)]
    pub organization: RecordId,
    pub is_active: bool,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Serialize)]
pub struct ProjectInsert {
    pub name: String,
    pub description: Option<String>,
    pub organization: RecordId,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct ProjectUpdate {
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

/// Pipeline database record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRecord {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub content: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Serialize)]
pub struct PipelineInsert {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct PipelineUpdate {
    pub content: String,
}

/// Job database record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    #[serde(skip_serializing)]
    pub id: RecordId,
    #[serde(skip_serializing)]
    pub pipeline_id: RecordId,
    pub status: String,
    pub content: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Serialize)]
pub struct JobInsert {
    pub pipeline_id: RecordId,
    pub status: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct JobUpdate {
    pub status: String,
}

/// User organization database record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrganizationRecord {
    #[serde(skip_serializing)]
    pub id: RecordId,
    #[serde(rename = "in", skip_serializing)]
    pub user_id: RecordId,
    #[serde(rename = "out", skip_serializing)]
    pub organization_id: RecordId,
    pub role: String,
    pub joined_at: Datetime,
}

#[derive(Debug, Serialize)]
pub struct UserOrganizationInsert {
    #[serde(rename = "in")]
    pub user_id: RecordId,
    #[serde(rename = "out")]
    pub organization_id: RecordId,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct UserOrganizationUpdate {
    pub role: String,
}

/// User project database record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProjectRecord {
    #[serde(skip_serializing)]
    pub id: RecordId,
    #[serde(rename = "in", skip_serializing)]
    pub user_id: RecordId,
    #[serde(rename = "out", skip_serializing)]
    pub project_id: RecordId,
    pub role: String,
    pub joined_at: Datetime,
}

#[derive(Debug, Serialize)]
pub struct UserProjectInsert {
    #[serde(rename = "in")]
    pub user_id: RecordId,
    #[serde(rename = "out")]
    pub project_id: RecordId,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct UserProjectUpdate {
    pub role: String,
}

/// Blacklist database record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistRecord {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub item: String,
    pub created_at: Datetime,
}
