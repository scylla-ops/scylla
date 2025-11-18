use crate::domain::DomainError;
use crate::domain::entities::Organization;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{Description, OrganizationId, OrganizationName};
use crate::infrastructure::persistence::OrganizationUpdate;
use crate::infrastructure::persistence::surrealdb::mappers::FromRecordId;
use crate::infrastructure::persistence::surrealdb::models::{
    OrganizationInsert, OrganizationRecord,
};
use chrono::DateTime;
use std::convert::{From, TryFrom};

impl TryFrom<OrganizationRecord> for Organization {
    type Error = DomainError;

    /// Convert database record to domain entity
    fn try_from(record: OrganizationRecord) -> DomainResult<Self> {
        let id = OrganizationId::from_record_id(record.id);
        let name = OrganizationName::new(record.name)?;
        let description = record.description.map(Description::new).transpose()?;

        Ok(Organization::new(
            id,
            name,
            description,
            record.is_active,
            DateTime::from(record.created_at),
            DateTime::from(record.updated_at),
        ))
    }
}

impl From<&Organization> for OrganizationInsert {
    /// Convert domain entity to insert record
    fn from(organization: &Organization) -> Self {
        OrganizationInsert {
            name: organization.name().as_str().to_string(),
            description: organization.description().map(|d| d.as_str().to_string()),
            is_active: organization.is_active(),
        }
    }
}

impl From<&Organization> for OrganizationUpdate {
    /// Convert domain entity to update record
    fn from(organization: &Organization) -> Self {
        OrganizationUpdate {
            name: organization.name().as_str().to_string(),
            description: organization.description().map(|d| d.as_str().to_string()),
            is_active: organization.is_active(),
        }
    }
}
