use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

// domain
#[derive(Serialize, Debug, Clone)]
pub struct InsertableOrganization {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct OrganizationPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

// db
#[derive(Serialize, Deserialize, Debug)]
pub struct Organization {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing)]
    pub is_active: bool,
    #[serde(skip_serializing)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserOrganizationRelation {
    #[serde(skip_serializing)]
    pub id: RecordId,
    #[serde(rename = "in")]
    pub user: RecordId,
    #[serde(rename = "out")]
    pub organization: RecordId,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}
