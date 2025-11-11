use crate::domain::entities::UserProject;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::UserProjectRepository;
use crate::domain::value_objects::{ProjectId, UserId, UserProjectId};
use crate::infrastructure::persistence::mappers::{FromRecordId, ToRecordId};
use crate::infrastructure::persistence::surrealdb::models::UserProjectRecord;
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// SurrealDB implementation of UserProjectRepository
#[derive(Constructor)]
pub struct SurrealUserProjectRepository {
    db: Arc<Surreal<Any>>,
}

#[async_trait]
impl UserProjectRepository for SurrealUserProjectRepository {
    async fn create(&self, user_project: &UserProject) -> DomainResult<UserProject> {
        let created: Option<UserProjectRecord> = self
            .db
            .query("RELATE $user_id->user_project->$project_id SET role = $role")
            .bind(("user_id", user_project.user_id().to_record_id()))
            .bind(("project_id", user_project.project_id().to_record_id()))
            .bind(("role", user_project.role().to_string()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        match created {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::infrastructure("Failed to create user project")),
        }
    }

    async fn find_by_id(&self, id: &UserProjectId) -> DomainResult<UserProject> {
        let result: Option<UserProjectRecord> = self
            .db
            .select(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match result {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::not_found("User project", id.to_string())),
        }
    }

    async fn find_by_user_and_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<UserProject> {
        let results: Vec<UserProjectRecord> = self
            .db
            .query("SELECT * FROM user_project WHERE in = $user_id AND out = $project_id")
            .bind(("user_id", user_id.to_record_id()))
            .bind(("project_id", project_id.to_record_id()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let result = results.into_iter().next();

        match result {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::not_found(
                "User project",
                format!(
                    "user_id: {}, project_id: {}",
                    user_id.to_string(),
                    project_id.to_string()
                ),
            )),
        }
    }

    async fn update(&self, user_project: &UserProject) -> DomainResult<UserProject> {
        let results: Vec<UserProjectRecord> = self
            .db
            .query("UPDATE user_project SET role = $role WHERE in = $user_id AND out = $project_id")
            .bind(("user_id", user_project.user_id().to_record_id()))
            .bind(("project_id", user_project.project_id().to_record_id()))
            .bind(("role", user_project.role().to_string()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let updated = results.into_iter().next();

        match updated {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::infrastructure("Failed to update user project")),
        }
    }

    async fn delete(&self, id: &UserProjectId) -> DomainResult<()> {
        self.db
            .delete::<Option<UserProjectRecord>>(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(&self) -> DomainResult<Vec<UserProject>> {
        let records: Vec<UserProjectRecord> = self
            .db
            .select("user_projects")
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        records
            .into_iter()
            .map(|record| record.try_into())
            .collect()
    }

    async fn list_projects_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<ProjectId>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM $user_id->user_project GROUP ALL")
            .bind(("user_id", user_id.to_record_id()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Get paginated records
        let records: Vec<UserProjectRecord> = self
            .db
            .query("SELECT * FROM $user_id->user_project LIMIT $limit START $start")
            .bind(("user_id", user_id.to_record_id()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let project_ids: DomainResult<Vec<ProjectId>> = records
            .into_iter()
            .map(|record| Ok(ProjectId::from_record_id(record.project_id)))
            .collect();

        Ok(PaginatedResult::new(project_ids?, &params, total_count))
    }

    async fn list_users_in_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<UserId>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM $project_id<-user_project GROUP ALL")
            .bind(("project_id", project_id.to_record_id()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Get paginated records
        let records: Vec<UserProjectRecord> = self
            .db
            .query("SELECT * FROM $project_id<-user_project LIMIT $limit START $start")
            .bind(("project_id", project_id.to_record_id()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let user_ids: DomainResult<Vec<UserId>> = records
            .into_iter()
            .map(|record| Ok(UserId::from_record_id(record.user_id)))
            .collect();

        Ok(PaginatedResult::new(user_ids?, &params, total_count))
    }

    async fn add_user_to_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        role: &str,
    ) -> DomainResult<UserProjectId> {
        let created: Option<UserProjectRecord> = self
            .db
            .query("RELATE $user_id->user_project->$project_id SET role = $role")
            .bind(("user_id", user_id.to_record_id()))
            .bind(("project_id", project_id.to_record_id()))
            .bind(("role", role.to_string()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        match created {
            Some(record) => Ok(UserProjectId::from_record_id(record.id)),
            None => Err(DomainError::infrastructure("Failed to add user to project")),
        }
    }

    async fn remove_user_from_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<()> {
        let _: Vec<UserProjectRecord> = self
            .db
            .query("DELETE user_project WHERE in = $user_id AND out = $project_id")
            .bind(("user_id", user_id.to_record_id()))
            .bind(("project_id", project_id.to_record_id()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(())
    }
}
