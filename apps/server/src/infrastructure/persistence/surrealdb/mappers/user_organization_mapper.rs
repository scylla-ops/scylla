use crate::domain::entities::UserOrganization;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::UserOrganizationRole;
use crate::domain::value_objects::{OrganizationId, UserId, UserOrganizationId};
use crate::infrastructure::persistence::mappers::ToRecordId;
use crate::infrastructure::persistence::surrealdb::mappers::FromRecordId;
use crate::infrastructure::persistence::surrealdb::models::{
    UserOrganizationInsert, UserOrganizationRecord, UserOrganizationUpdate,
};
use chrono::DateTime;

/// Mapper between UserOrganization domain entity and database records
pub struct UserOrganizationMapper;

impl UserOrganizationMapper {
    /// Convert database record to domain entity
    pub fn to_domain(record: UserOrganizationRecord) -> DomainResult<UserOrganization> {
        let id = UserOrganizationId::from_record_id(record.id);
        let user_id = UserId::from_record_id(record.user_id);
        let organization_id = OrganizationId::from_record_id(record.organization_id);
        let role = UserOrganizationRole::new(&record.role)?;

        UserOrganization::new(
            id,
            user_id,
            organization_id,
            role,
            DateTime::from(record.joined_at),
        )
    }

    /// Convert domain entity to insert record
    pub fn to_insert(user_organization: &UserOrganization) -> UserOrganizationInsert {
        UserOrganizationInsert {
            user_id: user_organization.user_id().to_record_id(),
            organization_id: user_organization.organization_id().to_record_id(),
            role: user_organization.role().to_string(),
        }
    }

    /// Convert domain entity to update record
    pub fn to_update(user_organization: &UserOrganization) -> UserOrganizationUpdate {
        UserOrganizationUpdate {
            role: user_organization.role().to_string(),
        }
    }
}
