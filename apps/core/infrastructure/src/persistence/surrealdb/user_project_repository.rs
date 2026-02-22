use domain::entities::{ProjectId, UserId, UserProject, UserProjectId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::UserProjectRepository;
use domain::value_objects::user_project::UserProjectRole;
use domain::value_objects::{PaginatedResult, PaginationParams};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;

pub struct SurrealUserProjectRepository {
    db: Arc<Surreal<Any>>,
}

impl SurrealUserProjectRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }
}

impl UserProjectRepository for SurrealUserProjectRepository {
    fn create(
        &self,
        user_project: &UserProject,
    ) -> impl Future<Output = DomainResult<UserProject>> + Send {
        let db = self.db.clone();
        let user_project = user_project.clone();
        async move {
            let created: Option<UserProject> = db
                .create(RecordId::new(
                    UserProjectId::table_name(),
                    user_project.id().as_str(),
                ))
                .content(user_project.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            created.ok_or_else(|| DomainError::infrastructure("Failed to create user project"))
        }
    }

    fn find_by_id(
        &self,
        id: &UserProjectId,
    ) -> impl Future<Output = DomainResult<UserProject>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            let result: Option<UserProject> = db
                .select(RecordId::new(UserProjectId::table_name(), id.as_str()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            result.ok_or_else(|| DomainError::not_found("UserProject", id.to_string()))
        }
    }

    fn find_by_user_and_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> impl Future<Output = DomainResult<UserProject>> + Send {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();
        let project_id_str = project_id.to_string();
        let table = UserProjectId::table_name().to_string();
        async move {
            let mut results: Vec<UserProject> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id AND project_id = $project_id LIMIT 1")
                .bind(("table", table))
                .bind(("user_id", user_id_str.clone()))
                .bind(("project_id", project_id_str.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            results.pop().ok_or_else(|| {
                DomainError::not_found(
                    "UserProject",
                    format!("user_id={}, project_id={}", user_id_str, project_id_str),
                )
            })
        }
    }

    fn update(
        &self,
        user_project: &UserProject,
    ) -> impl Future<Output = DomainResult<UserProject>> + Send {
        let db = self.db.clone();
        let user_project = user_project.clone();
        async move {
            let updated: Option<UserProject> = db
                .update(RecordId::new(
                    UserProjectId::table_name(),
                    user_project.id().as_str(),
                ))
                .content(user_project.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            updated
                .ok_or_else(|| DomainError::not_found("UserProject", user_project.id().to_string()))
        }
    }

    fn delete(&self, id: &UserProjectId) -> impl Future<Output = DomainResult<()>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            db.delete::<Option<UserProject>>(RecordId::new(
                UserProjectId::table_name(),
                id.as_str(),
            ))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(())
        }
    }

    fn list_all(&self) -> impl Future<Output = DomainResult<Vec<UserProject>>> + Send {
        let db = self.db.clone();
        let table = UserProjectId::table_name().to_string();
        async move {
            let results: Vec<UserProject> = db
                .query("SELECT * FROM type::table($table)")
                .bind(("table", table))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            Ok(results)
        }
    }

    fn list_projects_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<ProjectId>>> + Send {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserProjectId::table_name().to_string();
        async move {
            let count_result: Vec<i64> = db
                .query("SELECT count() FROM type::table($table) WHERE user_id = $user_id GROUP ALL")
                .bind(("table", table.clone()))
                .bind(("user_id", user_id_str.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take("count")
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let total_count = count_result.first().copied().unwrap_or(0) as u64;

            let results: Vec<UserProject> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id ORDER BY joined_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("user_id", user_id_str))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let project_ids: Vec<ProjectId> = results
                .into_iter()
                .map(|up| up.project_id().clone())
                .collect();

            Ok(PaginatedResult::new(project_ids, &params, total_count))
        }
    }

    fn list_users_in_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<UserId>>> + Send {
        let db = self.db.clone();
        let project_id_str = project_id.to_string();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserProjectId::table_name().to_string();
        async move {
            let count_result: Vec<i64> = db
                .query("SELECT count() FROM type::table($table) WHERE project_id = $project_id GROUP ALL")
                .bind(("table", table.clone()))
                .bind(("project_id", project_id_str.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take("count")
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let total_count = count_result.first().copied().unwrap_or(0) as u64;

            let results: Vec<UserProject> = db
                .query("SELECT * FROM type::table($table) WHERE project_id = $project_id ORDER BY joined_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("project_id", project_id_str))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let user_ids: Vec<UserId> =
                results.into_iter().map(|up| up.user_id().clone()).collect();

            Ok(PaginatedResult::new(user_ids, &params, total_count))
        }
    }

    fn add_user_to_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        role: &str,
    ) -> impl Future<Output = DomainResult<UserProjectId>> + Send {
        let db = self.db.clone();
        let user_id = user_id.clone();
        let project_id = project_id.clone();
        let role = role.to_string();
        async move {
            let parsed_role = UserProjectRole::new(&role)?;

            let user_project = UserProject::create(user_id, project_id, parsed_role)?;
            let id = user_project.id().clone();

            let _created: Option<UserProject> = db
                .create(RecordId::new(
                    UserProjectId::table_name(),
                    user_project.id().as_str(),
                ))
                .content(user_project)
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(id)
        }
    }

    fn remove_user_from_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> impl Future<Output = DomainResult<()>> + Send {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();
        let project_id_str = project_id.to_string();
        let table = UserProjectId::table_name().to_string();
        async move {
            db.query("DELETE FROM type::table($table) WHERE user_id = $user_id AND project_id = $project_id")
                .bind(("table", table))
                .bind(("user_id", user_id_str))
                .bind(("project_id", project_id_str))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(())
        }
    }
}
