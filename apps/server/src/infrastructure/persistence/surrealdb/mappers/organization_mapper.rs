use crate::domain::entities::Organization;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{Description, OrganizationId, OrganizationName};
use crate::infrastructure::persistence::OrganizationUpdate;
use crate::infrastructure::persistence::surrealdb::mappers::FromRecordId;
use crate::infrastructure::persistence::surrealdb::models::{
    OrganizationInsert, OrganizationRecord,
};
use chrono::DateTime;

/// Mapper between Organization domain entity and database records
pub struct OrganizationMapper;

impl OrganizationMapper {
    /// Convert database record to domain entity
    pub fn to_domain(record: OrganizationRecord) -> DomainResult<Organization> {
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

    /// Convert domain entity to insert record
    pub fn to_insert(organization: &Organization) -> OrganizationInsert {
        OrganizationInsert {
            name: organization.name().as_str().to_string(),
            description: organization.description().map(|d| d.as_str().to_string()),
            is_active: organization.is_active(),
        }
    }

    /// Convert domain entity to update record
    pub fn to_update(organization: &Organization) -> OrganizationUpdate {
        OrganizationUpdate {
            name: organization.name().as_str().to_string(),
            description: organization.description().map(|d| d.as_str().to_string()),
            is_active: organization.is_active(),
        }
    }
}
