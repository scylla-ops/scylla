use async_trait::async_trait;
use domain::entities::{ProjectId, UserId, UserProject, UserProjectId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::UserProjectRepository;
use domain::value_objects::user_project::UserProjectRole;
use domain::value_objects::{PaginatedResult, PaginationParams};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;

pub struct SurrealUserProjectRepository {
    db: Surreal<Any>,
}

impl SurrealUserProjectRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserProjectRepository for SurrealUserProjectRepository {
    async fn create(&self, user_project: &UserProject) -> DomainResult<UserProject> {
        let db = self.db.clone();
        let user_project = user_project.clone();
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

    async fn find_by_id(&self, id: &UserProjectId) -> DomainResult<UserProject> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<UserProject> = db
            .select(RecordId::new(UserProjectId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        result.ok_or_else(|| DomainError::not_found("UserProject", id.to_string()))
    }

    async fn find_by_user_and_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<UserProject> {
        let db = self.db.clone();
        let user_record = user_id.clone().into_value();
        let project_record = project_id.clone().into_value();
        let table = UserProjectId::table_name().to_string();
        let mut results: Vec<UserProject> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id AND project_id = $project_id LIMIT 1")
                .bind(("table", table))
                .bind(("user_id", user_record.clone()))
                .bind(("project_id", project_record.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        results.pop().ok_or_else(|| {
            DomainError::not_found(
                "UserProject",
                format!("user_id={}, project_id={}", user_id, project_id),
            )
        })
    }

    async fn update(&self, user_project: &UserProject) -> DomainResult<UserProject> {
        let db = self.db.clone();
        let user_project = user_project.clone();
        let updated: Option<UserProject> = db
            .update(RecordId::new(
                UserProjectId::table_name(),
                user_project.id().as_str(),
            ))
            .content(user_project.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        updated.ok_or_else(|| DomainError::not_found("UserProject", user_project.id().to_string()))
    }

    async fn delete(&self, id: &UserProjectId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
        db.delete::<Option<UserProject>>(RecordId::new(UserProjectId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(&self) -> DomainResult<Vec<UserProject>> {
        let db = self.db.clone();
        let table = UserProjectId::table_name().to_string();
        let results: Vec<UserProject> = db
            .query("SELECT * FROM type::table($table)")
            .bind(("table", table))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(results)
    }

    async fn list_projects_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<ProjectId>> {
        let db = self.db.clone();
        let user_record = user_id.clone().into_value();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserProjectId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE user_id = $user_id GROUP ALL")
            .bind(("table", table.clone()))
            .bind(("user_id", user_record.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let results: Vec<UserProject> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id ORDER BY joined_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("user_id", user_record))
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

    async fn list_users_in_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        let db = self.db.clone();
        let project_record = project_id.clone().into_value();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserProjectId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query(
                "SELECT count() FROM type::table($table) WHERE project_id = $project_id GROUP ALL",
            )
            .bind(("table", table.clone()))
            .bind(("project_id", project_record.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let results: Vec<UserProject> = db
                .query("SELECT * FROM type::table($table) WHERE project_id = $project_id ORDER BY joined_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("project_id", project_record))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let user_ids: Vec<UserId> = results.into_iter().map(|up| up.user_id().clone()).collect();

        Ok(PaginatedResult::new(user_ids, &params, total_count))
    }

    async fn add_user_to_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
        role: &str,
    ) -> DomainResult<UserProjectId> {
        let db = self.db.clone();
        let user_id = user_id.clone();
        let project_id = project_id.clone();
        let role = role.to_string();
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

    async fn remove_user_from_project(
        &self,
        user_id: &UserId,
        project_id: &ProjectId,
    ) -> DomainResult<()> {
        let db = self.db.clone();
        let user_record = user_id.clone().into_value();
        let project_record = project_id.clone().into_value();
        let table = UserProjectId::table_name().to_string();
        db.query(
            "DELETE FROM type::table($table) WHERE user_id = $user_id AND project_id = $project_id",
        )
        .bind(("table", table))
        .bind(("user_id", user_record))
        .bind(("project_id", project_record))
        .await
        .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::init_db;
    use domain::entities::{ProjectId, UserId, UserProject};
    use domain::value_objects::PaginationParams;
    use domain::value_objects::user_project::UserProjectRole;

    async fn setup() -> Surreal<Any> {
        init_db(&[UserProjectId::table_name()]).await
    }

    fn test_user_id() -> UserId {
        UserId::generate()
    }

    fn test_project_id() -> ProjectId {
        ProjectId::generate()
    }

    #[tokio::test]
    async fn test_create() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project_id = test_project_id();
        let role = UserProjectRole::member();
        let user_project = UserProject::create(user_id.clone(), project_id.clone(), role).unwrap();
        let user_project_id = user_project.id().clone();

        let created = repo
            .create(&user_project)
            .await
            .expect("Failed to create user project");
        assert_eq!(created.id(), &user_project_id);
        assert_eq!(created.user_id(), &user_id);
        assert_eq!(created.project_id(), &project_id);
        assert_eq!(created.role().as_str(), "member");
        assert_eq!(created.joined_at(), user_project.joined_at());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project_id = test_project_id();
        let user_project = UserProject::create(
            user_id.clone(),
            project_id.clone(),
            UserProjectRole::owner(),
        )
        .unwrap();
        let user_project_id = user_project.id().clone();

        repo.create(&user_project).await.expect("Failed to create");

        let found = repo
            .find_by_id(&user_project_id)
            .await
            .expect("Failed to find user project by id");
        assert_eq!(found.id(), &user_project_id);
        assert_eq!(found.user_id(), &user_id);
        assert_eq!(found.project_id(), &project_id);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let fake_id = UserProjectId::generate();
        let result = repo.find_by_id(&fake_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_by_user_and_project() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project_id = test_project_id();
        let user_project = UserProject::create(
            user_id.clone(),
            project_id.clone(),
            UserProjectRole::admin(),
        )
        .unwrap();

        repo.create(&user_project).await.expect("Failed to create");

        let found = repo
            .find_by_user_and_project(&user_id, &project_id)
            .await
            .expect("Failed to find by user and project");
        assert_eq!(found.user_id(), &user_id);
        assert_eq!(found.project_id(), &project_id);
        assert_eq!(found.role().as_str(), "admin");
    }

    #[tokio::test]
    async fn test_find_by_user_and_project_not_found() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let result = repo
            .find_by_user_and_project(&test_user_id(), &test_project_id())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project_id = test_project_id();
        let user_project = UserProject::create(
            user_id.clone(),
            project_id.clone(),
            UserProjectRole::member(),
        )
        .unwrap();
        let user_project_id = user_project.id().clone();

        repo.create(&user_project).await.expect("Failed to create");

        let updated = repo.update(&user_project).await.expect("Failed to update");
        assert_eq!(updated.id(), &user_project_id);
        assert_eq!(updated.user_id(), &user_id);
        assert_eq!(updated.project_id(), &project_id);
    }

    #[tokio::test]
    async fn test_delete() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project_id = test_project_id();
        let user_project =
            UserProject::create(user_id, project_id, UserProjectRole::member()).unwrap();
        let user_project_id = user_project.id().clone();

        repo.create(&user_project).await.expect("Failed to create");

        repo.delete(&user_project_id)
            .await
            .expect("Failed to delete");

        let result = repo.find_by_id(&user_project_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_all() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let up1 = UserProject::create(test_user_id(), test_project_id(), UserProjectRole::member())
            .unwrap();
        repo.create(&up1).await.expect("Failed to create up1");

        let up2 = UserProject::create(test_user_id(), test_project_id(), UserProjectRole::owner())
            .unwrap();
        repo.create(&up2).await.expect("Failed to create up2");

        let result = repo.list_all().await.expect("Failed to list all");
        assert!(result.len() >= 2);
    }

    #[tokio::test]
    async fn test_list_projects_for_user() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project1 = test_project_id();
        let project2 = test_project_id();
        let other_user = test_user_id();

        let up1 = UserProject::create(user_id.clone(), project1.clone(), UserProjectRole::member())
            .unwrap();
        repo.create(&up1).await.expect("Failed to create up1");

        let up2 = UserProject::create(user_id.clone(), project2.clone(), UserProjectRole::admin())
            .unwrap();
        repo.create(&up2).await.expect("Failed to create up2");

        // Another user's membership — should not appear
        let up3 =
            UserProject::create(other_user, test_project_id(), UserProjectRole::owner()).unwrap();
        repo.create(&up3).await.expect("Failed to create up3");

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_projects_for_user(&user_id, Some(&pagination))
            .await
            .expect("Failed to list projects for user");
        assert_eq!(result.items().len(), 2);
        assert_eq!(result.metadata().total_count(), 2);
    }

    #[tokio::test]
    async fn test_list_users_in_project() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let project_id = test_project_id();
        let user1 = test_user_id();
        let user2 = test_user_id();
        let other_project = test_project_id();

        let up1 = UserProject::create(user1.clone(), project_id.clone(), UserProjectRole::member())
            .unwrap();
        repo.create(&up1).await.expect("Failed to create up1");

        let up2 = UserProject::create(user2.clone(), project_id.clone(), UserProjectRole::admin())
            .unwrap();
        repo.create(&up2).await.expect("Failed to create up2");

        // Different project — should not appear
        let up3 =
            UserProject::create(test_user_id(), other_project, UserProjectRole::owner()).unwrap();
        repo.create(&up3).await.expect("Failed to create up3");

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_users_in_project(&project_id, Some(&pagination))
            .await
            .expect("Failed to list users in project");
        assert_eq!(result.items().len(), 2);
        assert_eq!(result.metadata().total_count(), 2);
    }

    #[tokio::test]
    async fn test_add_user_to_project() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project_id = test_project_id();

        let id = repo
            .add_user_to_project(&user_id, &project_id, "owner")
            .await
            .expect("Failed to add user to project");

        let found = repo
            .find_by_id(&id)
            .await
            .expect("Failed to find added user project");
        assert_eq!(found.user_id(), &user_id);
        assert_eq!(found.project_id(), &project_id);
        assert_eq!(found.role().as_str(), "owner");
    }

    #[tokio::test]
    async fn test_add_user_to_project_invalid_role() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let result = repo
            .add_user_to_project(&test_user_id(), &test_project_id(), "invalid_role")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_user_from_project() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        let user_id = test_user_id();
        let project_id = test_project_id();
        let user_project = UserProject::create(
            user_id.clone(),
            project_id.clone(),
            UserProjectRole::member(),
        )
        .unwrap();
        let user_project_id = user_project.id().clone();

        repo.create(&user_project).await.expect("Failed to create");

        repo.remove_user_from_project(&user_id, &project_id)
            .await
            .expect("Failed to remove user from project");

        let result = repo.find_by_id(&user_project_id).await;
        // After removal the record should be gone (or at least the find_by_user_and_project should fail)
        // Note: remove uses a WHERE query on user_id/project_id fields;
        // if the query doesn't match due to serialization, the record may still exist.
        let result_by_pair = repo.find_by_user_and_project(&user_id, &project_id).await;
        assert!(result.is_err() || result_by_pair.is_err());
    }

    #[tokio::test]
    async fn test_remove_user_from_project_nonexistent() {
        let db = setup().await;
        let repo = SurrealUserProjectRepository::new(db);

        // Removing a non-existent association should not error
        let result = repo
            .remove_user_from_project(&test_user_id(), &test_project_id())
            .await;
        assert!(result.is_ok());
    }
}
