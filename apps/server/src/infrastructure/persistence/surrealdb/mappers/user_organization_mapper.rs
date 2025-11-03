use crate::domain::DomainError;
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
use std::convert::{From, TryFrom};

impl TryFrom<UserOrganizationRecord> for UserOrganization {
    type Error = DomainError;

    /// Convert database record to domain entity
    fn try_from(record: UserOrganizationRecord) -> DomainResult<Self> {
        let id = UserOrganizationId::from_record_id(record.id);
        let user_id = UserId::from_record_id(record.user_id);
        let organization_id = OrganizationId::from_record_id(record.organization_id);
        let role = UserOrganizationRole::new(&record.role)?;

        Ok(UserOrganization::new(
            id,
            user_id,
            organization_id,
            role,
            DateTime::from(record.joined_at),
        ))
    }
}

impl From<&UserOrganization> for UserOrganizationInsert {
    /// Convert domain entity to insert record
    fn from(user_organization: &UserOrganization) -> Self {
        UserOrganizationInsert {
            user_id: user_organization.user_id().to_record_id(),
            organization_id: user_organization.organization_id().to_record_id(),
            role: user_organization.role().to_string(),
        }
    }
}

impl From<&UserOrganization> for UserOrganizationUpdate {
    /// Convert domain entity to update record
    fn from(user_organization: &UserOrganization) -> Self {
        UserOrganizationUpdate {
            role: user_organization.role().to_string(),
        }
    }
}
