use domain::entities::{Project, ProjectId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::ProjectRepository;
use domain::value_objects::{PaginatedResult, PaginationParams};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use async_trait::async_trait;

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
        let db = self.db.clone();
        let project = project.clone();
            let created: Option<Project> = db
                .create(RecordId::new(ProjectId::table_name(), project.id().as_str()))
                .content(project.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            created
                .ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
    }

    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        let db = self.db.clone();
        let id = id.clone();
            let result: Option<Project> = db
                .select(RecordId::new(ProjectId::table_name(), id.as_str()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            result.ok_or_else(|| DomainError::not_found("Project", id.to_string()))
    }

    async fn update(&self, project: &Project) -> DomainResult<Project> {
        let db = self.db.clone();
        let project = project.clone();
            let updated: Option<Project> = db
                .update(RecordId::new(ProjectId::table_name(), project.id().as_str()))
                .content(project.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            updated.ok_or_else(|| DomainError::not_found("Project", project.id().to_string()))
    }

    async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
            db.delete::<Option<Project>>(RecordId::new(ProjectId::table_name(), id.as_str()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = ProjectId::table_name().to_string();
            let count_result: Vec<i64> = db
                .query("SELECT count() FROM type::table($table) GROUP ALL")
                .bind(("table", table.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take("count")
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let total_count = count_result.first().copied().unwrap_or(0) as u64;

            let projects: Vec<Project> = db
                .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            Ok(PaginatedResult::new(projects, &params, total_count))
    }

    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = ProjectId::table_name().to_string();
            let count_result: Vec<i64> = db
                .query("SELECT count() FROM type::table($table) WHERE is_active = true GROUP ALL")
                .bind(("table", table.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take("count")
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let total_count = count_result.first().copied().unwrap_or(0) as u64;

            let projects: Vec<Project> = db
                .query("SELECT * FROM type::table($table) WHERE is_active = true ORDER BY created_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            Ok(PaginatedResult::new(projects, &params, total_count))
    }
}
