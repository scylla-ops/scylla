use crate::domain::entities::UserProject;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::UserProjectRole;
use crate::domain::value_objects::{ProjectId, UserId, UserProjectId};
use crate::infrastructure::persistence::UserProjectUpdate;
use crate::infrastructure::persistence::surrealdb::mappers::{FromRecordId, ToRecordId};
use crate::infrastructure::persistence::surrealdb::models::{UserProjectInsert, UserProjectRecord};
use chrono::DateTime;

/// Mapper between UserProject domain entity and database records
pub struct UserProjectMapper;

impl UserProjectMapper {
    /// Convert database record to domain entity
    pub fn to_domain(record: UserProjectRecord) -> DomainResult<UserProject> {
        let id = UserProjectId::from_record_id(record.id);
        let user_id = UserId::from_record_id(record.user_id);
        let project_id = ProjectId::from_record_id(record.project_id);
        let role = UserProjectRole::new(&record.role)?;

        Ok(UserProject::new(
            id,
            user_id,
            project_id,
            role,
            DateTime::from(record.joined_at),
        ))
    }

    /// Convert domain entity to insert record
    pub fn to_insert(user_project: &UserProject) -> UserProjectInsert {
        UserProjectInsert {
            user_id: user_project.user_id().to_record_id(),
            project_id: user_project.project_id().to_record_id(),
            role: user_project.role().to_string(),
        }
    }

    /// Convert domain entity to update record
    pub fn to_update(user_project: &UserProject) -> UserProjectUpdate {
        UserProjectUpdate {
            role: user_project.role().to_string(),
        }
    }
}
