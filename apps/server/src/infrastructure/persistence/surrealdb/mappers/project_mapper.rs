use crate::domain::entities::Project;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::{Description, OrganizationId, ProjectId, ProjectName};
use crate::infrastructure::persistence::ProjectUpdate;
use crate::infrastructure::persistence::mappers::ToRecordId;
use crate::infrastructure::persistence::surrealdb::models::{ProjectInsert, ProjectRecord};
use chrono::DateTime;

/// Mapper between Project domain entity and database records
pub struct ProjectMapper;

impl ProjectMapper {
    /// Convert database record to domain entity
    pub fn to_domain(record: ProjectRecord) -> DomainResult<Project> {
        let id = ProjectId::new(record.id.key().to_string());
        let organization_id = OrganizationId::new(record.organization.key().to_string());
        let name = ProjectName::new(record.name)?;
        let description = record.description.map(Description::new).transpose()?;

        Ok(Project::new(
            id,
            name,
            description,
            organization_id,
            record.is_active,
            DateTime::from(record.created_at),
            DateTime::from(record.updated_at),
        ))
    }

    /// Convert domain entity to insert record
    pub fn to_insert(project: &Project) -> ProjectInsert {
        ProjectInsert {
            name: project.name().as_str().to_string(),
            description: project.description().map(|d| d.as_str().to_string()),
            organization: project.organization_id().to_record_id(),
            is_active: project.is_active(),
        }
    }

    /// Convert domain entity to update record
    pub fn to_update(project: &Project) -> ProjectUpdate {
        ProjectUpdate {
            name: project.name().as_str().to_string(),
            description: project.description().map(|d| d.as_str().to_string()),
            is_active: project.is_active(),
        }
    }

    // pub fn to_update(project: &Project) -> ProjectRecord {
    //     ProjectRecord {
    //         id: project.id().to_record_id(),
    //         name: project.name().as_str().to_string(),
    //         description: project.description().map(|d| d.as_str().to_string()),
    //         organization: project.organization_id().to_record_id(),
    //         is_active: project.is_active(),
    //         created_at: project.created_at(),
    //         updated_at: project.updated_at(),
    //     }
    // }
}
