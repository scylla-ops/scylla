use crate::domain::entities::Project;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::ProjectRepository;
use crate::domain::value_objects::ProjectId;
use crate::infrastructure::persistence::mappers::ToRecordId;
use crate::infrastructure::persistence::surrealdb::mappers::ProjectMapper;
use crate::infrastructure::persistence::surrealdb::models::ProjectRecord;
use async_trait::async_trait;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// SurrealDB implementation of ProjectRepository
pub struct SurrealProjectRepository {
    db: Arc<Surreal<Any>>,
}

impl SurrealProjectRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProjectRepository for SurrealProjectRepository {
    async fn create(&self, project: &Project) -> DomainResult<Project> {
        let insert = ProjectMapper::to_insert(project);
        let created: Option<ProjectRecord> = self
            .db
            .create("projects")
            .content(insert)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match created {
            Some(record) => Ok(ProjectMapper::to_domain(record)?),
            None => Err(DomainError::infrastructure("Failed to create project")),
        }
    }

    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        let result: Option<ProjectRecord> = self
            .db
            .select(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match result {
            Some(record) => Ok(ProjectMapper::to_domain(record)?),
            None => Err(DomainError::not_found("Project", id.to_string())),
        }
    }

    async fn update(&self, project: &Project) -> DomainResult<Project> {
        let record = ProjectMapper::to_update(project);
        let updated: Option<ProjectRecord> = self
            .db
            .update(project.id().to_record_id())
            .merge(record)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match updated {
            Some(record) => Ok(ProjectMapper::to_domain(record)?),
            None => Err(DomainError::infrastructure("Failed to update project")),
        }
    }

    async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        self.db
            .delete::<Option<ProjectRecord>>(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<Project>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM projects GROUP ALL")
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
        let records: Vec<ProjectRecord> = self
            .db
            .query("SELECT * FROM projects ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let projects: DomainResult<Vec<Project>> = records
            .into_iter()
            .map(|record| ProjectMapper::to_domain(record))
            .collect();

        Ok(PaginatedResult::new(projects?, &params, total_count))
    }

    async fn list_active(
        &self,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<Project>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM projects WHERE is_active = true GROUP ALL")
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
        let records: Vec<ProjectRecord> = self
            .db
            .query("SELECT * FROM projects WHERE is_active = true ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let projects: DomainResult<Vec<Project>> = records
            .into_iter()
            .map(|record| ProjectMapper::to_domain(record))
            .collect();

        Ok(PaginatedResult::new(projects?, &params, total_count))
    }
}
