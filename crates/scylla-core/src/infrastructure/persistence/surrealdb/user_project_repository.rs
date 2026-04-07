use crate::application::ports::UserProjectRepository;
use crate::domain::entities::{ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb_types::SurrealValue;
use tracing::instrument;

pub struct SurrealUserProjectRepository {
    db: Surreal<Any>,
}

impl SurrealUserProjectRepository {
    #[must_use]
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserProjectRepository for SurrealUserProjectRepository {
    #[instrument(skip(self), fields(user_id = %user_id, project_id = %project_id))]
    async fn add_member(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<()> {
        let user_record = user_id.clone().into_value();
        let project_record = project_id.clone().into_value();

        self.db
            .query("RELATE $user_id -> user_project -> $project_id")
            .bind(("user_id", user_record))
            .bind(("project_id", project_record))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        Ok(())
    }

    #[instrument(skip(self), fields(user_id = %user_id, project_id = %project_id))]
    async fn remove_member(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<()> {
        let user_record = user_id.clone().into_value();
        let project_record = project_id.clone().into_value();

        self.db
            .query("DELETE FROM user_project WHERE in = $user_id AND out = $project_id")
            .bind(("user_id", user_record))
            .bind(("project_id", project_record))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        Ok(())
    }

    #[instrument(skip(self), fields(user_id = %user_id, project_id = %project_id))]
    async fn is_member(&self, user_id: &UserId, project_id: &ProjectId) -> DomainResult<bool> {
        let user_record = user_id.clone().into_value();
        let project_record = project_id.clone().into_value();

        let count_result: Vec<i64> = self
            .db
            .query("SELECT count() FROM user_project WHERE in = $user_id AND out = $project_id GROUP ALL")
            .bind(("user_id", user_record))
            .bind(("project_id", project_record))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(count_result.first().copied().unwrap_or(0) > 0)
    }

    #[instrument(skip(self), fields(project_id = %project_id))]
    async fn list_members(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        let project_record = project_id.clone().into_value();
        let params = pagination.copied().unwrap_or_default();

        let count_result: Vec<i64> = self
            .db
            .query("SELECT count() FROM user_project WHERE out = $project_id GROUP ALL")
            .bind(("project_id", project_record.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let results: Vec<UserId> = self
            .db
            .query("SELECT VALUE in FROM user_project WHERE out = $project_id LIMIT $limit START $start")
            .bind(("project_id", project_record))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(results, &params, total_count))
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn list_user_projects(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<ProjectId>> {
        let user_record = user_id.clone().into_value();
        let params = pagination.copied().unwrap_or_default();

        let count_result: Vec<i64> = self
            .db
            .query("SELECT count() FROM user_project WHERE in = $user_id GROUP ALL")
            .bind(("user_id", user_record.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let results: Vec<ProjectId> = self
            .db
            .query(
                "SELECT VALUE out FROM user_project WHERE in = $user_id LIMIT $limit START $start",
            )
            .bind(("user_id", user_record))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(results, &params, total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{ProjectId, UserId};
    use crate::domain::value_objects::PaginationParams;
    use crate::infrastructure::test_utils::init_db;

    async fn setup() -> Surreal<Any> {
        let db = init_db(&[]).await;
        db.query("DEFINE TABLE IF NOT EXISTS user_project TYPE RELATION IN users OUT projects SCHEMALESS")
            .await
            .unwrap()
            .check()
            .unwrap();
        db
    }

    fn test_user_id() -> UserId {
        UserId::generate()
    }

    fn test_project_id() -> ProjectId {
        ProjectId::generate()
    }

    #[tokio::test]
    async fn test_add_member() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project_id = test_project_id();

        repo.add_member(&user_id, &project_id)
            .await
            .expect("Failed to add member");

        let is_member = repo
            .is_member(&user_id, &project_id)
            .await
            .expect("Failed to check membership");
        assert!(is_member);
    }

    #[tokio::test]
    async fn test_remove_member() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project_id = test_project_id();

        repo.add_member(&user_id, &project_id).await.unwrap();

        repo.remove_member(&user_id, &project_id)
            .await
            .expect("Failed to remove member");

        let is_member = repo.is_member(&user_id, &project_id).await.unwrap();
        assert!(!is_member);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_member() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let result = repo
            .remove_member(&test_user_id(), &test_project_id())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_is_member_false() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let is_member = repo
            .is_member(&test_user_id(), &test_project_id())
            .await
            .unwrap();
        assert!(!is_member);
    }

    #[tokio::test]
    async fn test_list_members_empty() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let project_id = test_project_id();
        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_members(&project_id, Some(&pagination))
            .await
            .unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_user_projects_empty() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_user_projects(&user_id, Some(&pagination))
            .await
            .unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_members() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let project_id = test_project_id();
        let user1 = test_user_id();
        let user2 = test_user_id();
        let other_project = test_project_id();

        repo.add_member(&user1, &project_id).await.unwrap();
        repo.add_member(&user2, &project_id).await.unwrap();
        repo.add_member(&test_user_id(), &other_project)
            .await
            .unwrap();

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_members(&project_id, Some(&pagination))
            .await
            .expect("Failed to list members");
        assert_eq!(result.items().len(), 2);
        assert_eq!(result.metadata().total_count(), 2);
    }

    #[tokio::test]
    async fn test_list_user_projects() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project1 = test_project_id();
        let project2 = test_project_id();
        let other_user = test_user_id();

        repo.add_member(&user_id, &project1).await.unwrap();
        repo.add_member(&user_id, &project2).await.unwrap();
        repo.add_member(&other_user, &test_project_id())
            .await
            .unwrap();

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_user_projects(&user_id, Some(&pagination))
            .await
            .expect("Failed to list user projects");
        assert_eq!(result.items().len(), 2);
        assert_eq!(result.metadata().total_count(), 2);
    }
}
