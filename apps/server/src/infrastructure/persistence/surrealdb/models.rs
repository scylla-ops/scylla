use crate::domain::entities::{JobNodeExecution, PipelineNode};
use crate::domain::value_objects::JobStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub name: String,
    pub nodes: Vec<PipelineNode>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Serialize)]
pub struct PipelineInsert {
    pub name: String,
    pub nodes: Vec<PipelineNode>,
}

#[derive(Debug, Serialize)]
pub struct PipelineUpdate {
    pub name: String,
    pub nodes: Vec<PipelineNode>,
}

/// Job database record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub pipeline_id: String,
    pub status: JobStatus,
    pub executions: HashMap<String, JobNodeExecutionRecord>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobNodeExecutionRecord {
    pub node_id: String,
    pub state: JobStatus,
    pub started_at: Option<Datetime>,
    pub finished_at: Option<Datetime>,
}

#[derive(Debug, Serialize)]
pub struct JobInsert {
    pub pipeline_id: String,
    pub status: JobStatus,
    pub executions: HashMap<String, JobNodeExecutionRecord>,
}

#[derive(Debug, Serialize)]
pub struct JobUpdate {
    pub status: JobStatus,
    pub executions: HashMap<String, JobNodeExecutionRecord>,
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
