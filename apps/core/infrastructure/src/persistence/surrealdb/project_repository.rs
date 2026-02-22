use domain::entities::{Project, ProjectId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::ProjectRepository;
use domain::value_objects::{PaginatedResult, PaginationParams};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;

pub struct SurrealProjectRepository {
    db: Arc<Surreal<Any>>,
}

impl SurrealProjectRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }
}

impl ProjectRepository for SurrealProjectRepository {
    fn create(&self, project: &Project) -> impl Future<Output = DomainResult<Project>> + Send {
        let db = self.db.clone();
        let project = project.clone();
        async move {
            let created: Option<Project> = db
                .create(RecordId::new(ProjectId::table_name(), project.id().as_str()))
                .content(project.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            created
                .ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
        }
    }

    fn find_by_id(&self, id: &ProjectId) -> impl Future<Output = DomainResult<Project>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            let result: Option<Project> = db
                .select(RecordId::new(ProjectId::table_name(), id.as_str()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            result.ok_or_else(|| DomainError::not_found("Project", id.to_string()))
        }
    }

    fn update(&self, project: &Project) -> impl Future<Output = DomainResult<Project>> + Send {
        let db = self.db.clone();
        let project = project.clone();
        async move {
            let updated: Option<Project> = db
                .update(RecordId::new(ProjectId::table_name(), project.id().as_str()))
                .content(project.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            updated.ok_or_else(|| DomainError::not_found("Project", project.id().to_string()))
        }
    }

    fn delete(&self, id: &ProjectId) -> impl Future<Output = DomainResult<()>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            db.delete::<Option<Project>>(RecordId::new(ProjectId::table_name(), id.as_str()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(())
        }
    }

    fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<Project>>> + Send {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = ProjectId::table_name().to_string();
        async move {
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
    }

    fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<Project>>> + Send {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = ProjectId::table_name().to_string();
        async move {
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
}
