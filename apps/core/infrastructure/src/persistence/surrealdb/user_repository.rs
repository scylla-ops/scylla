use async_trait::async_trait;
use domain::entities::{User, UserId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::UserRepository;
use domain::value_objects::user::UserName;
use domain::value_objects::{PaginatedResult, PaginationParams};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;

pub struct SurrealUserRepository {
    db: Surreal<Any>,
}

impl SurrealUserRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for SurrealUserRepository {
    async fn create(&self, user: &User) -> DomainResult<User> {
        let db = self.db.clone();
        let user = user.clone();
        let user_id = user.id().clone().into_value();
        let created: Option<User> = db
            .create(RecordId::from_value(user_id).unwrap())
            .content(user.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        created.ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
    }

    async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<User> = db
            .select(RecordId::new(UserId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        result.ok_or_else(|| DomainError::not_found("User", id.to_string()))
    }

    async fn find_by_username(&self, username: &UserName) -> DomainResult<User> {
        let db = self.db.clone();
        let table = UserId::table_name().to_string();
        let mut results: Vec<User> = db
            .query("SELECT * FROM type::table($table) WHERE username = $username LIMIT 1")
            .bind(("table", table))
            .bind(("username", username.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        results
            .pop()
            .ok_or_else(|| DomainError::not_found("User", username.as_str()))
    }

    async fn update(&self, user: &User) -> DomainResult<User> {
        let db = self.db.clone();
        let user = user.clone();
        let updated: Option<User> = db
            .update(RecordId::new(UserId::table_name(), user.id().as_str()))
            .content(user.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        updated.ok_or_else(|| DomainError::not_found("User", user.id().to_string()))
    }

    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
        db.delete::<Option<User>>(RecordId::new(UserId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) GROUP ALL")
            .bind(("table", table.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let users: Vec<User> = db
                .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(PaginatedResult::new(users, &params, total_count))
    }

    async fn username_exists(&self, username: &UserName) -> DomainResult<bool> {
        let db = self.db.clone();
        let table = UserId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE username = $username GROUP ALL")
            .bind(("table", table))
            .bind(("username", username.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let count = count_result.first().copied().unwrap_or(0);
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::init_db;
    use domain::entities::User;
    use domain::value_objects::PaginationParams;
    use domain::value_objects::user::UserName;

    async fn setup() -> Surreal<Any> {
        init_db(&[UserId::table_name()]).await
    }

    #[tokio::test]
    async fn test_create() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username = UserName::new("testuser").expect("Invalid username");
        let user = User::create(username, "hashed_password_123".to_string());
        let user_id = user.id().clone();

        let created = repo.create(&user).await.expect("Failed to create user");
        assert_eq!(created.id(), &user_id);
        assert_eq!(created.username(), user.username());
        assert_eq!(created.password_hash(), user.password_hash());
        assert_eq!(created.is_active(), user.is_active());
        assert_eq!(created.created_at(), user.created_at());
        assert_eq!(created.updated_at(), user.updated_at());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username = UserName::new("find_by_id_user").expect("Invalid username");
        let user = User::create(username, "hashed_password".to_string());
        let user_id = user.id().clone();

        repo.create(&user).await.expect("Failed to create");

        let found = repo
            .find_by_id(&user_id)
            .await
            .expect("Failed to find user by id");
        assert_eq!(found.id(), &user_id);
        assert_eq!(found.username(), user.username());
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let fake_id = UserId::generate();
        let result = repo.find_by_id(&fake_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_by_username() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username = UserName::new("find_by_username_user").expect("Invalid username");
        let user = User::create(username.clone(), "hashed_password".to_string());
        let user_id = user.id().clone();

        repo.create(&user).await.expect("Failed to create");

        let found = repo
            .find_by_username(&username)
            .await
            .expect("Failed to find user by username");
        assert_eq!(found.id(), &user_id);
        assert_eq!(found.username(), &username);
    }

    #[tokio::test]
    async fn test_find_by_username_not_found() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username = UserName::new("non_existent_user").expect("Invalid username");
        let result = repo.find_by_username(&username).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username = UserName::new("update_user").expect("Invalid username");
        let mut user = User::create(username, "hashed_password".to_string());
        let user_id = user.id().clone();

        repo.create(&user).await.expect("Failed to create");

        let new_username = UserName::new("updated_username").expect("Invalid username");
        user.update_username(new_username.clone()).unwrap();
        user.update_password_hash("new_hashed_password".to_string())
            .unwrap();

        let updated = repo.update(&user).await.expect("Failed to update");
        assert_eq!(updated.id(), &user_id);
        assert_eq!(updated.username(), &new_username);
        assert_eq!(updated.password_hash(), "new_hashed_password");
    }

    #[tokio::test]
    async fn test_delete() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username = UserName::new("delete_user").expect("Invalid username");
        let user = User::create(username, "hashed_password".to_string());
        let user_id = user.id().clone();

        repo.create(&user).await.expect("Failed to create");

        repo.delete(&user_id).await.expect("Failed to delete");

        let result = repo.find_by_id(&user_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_all() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username1 = UserName::new("list_all_user_1").expect("Invalid username");
        let user1 = User::create(username1, "hash1".to_string());
        repo.create(&user1).await.expect("Failed to create user1");

        let username2 = UserName::new("list_all_user_2").expect("Invalid username");
        let user2 = User::create(username2, "hash2".to_string());
        repo.create(&user2).await.expect("Failed to create user2");

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_all(Some(&pagination))
            .await
            .expect("Failed to list all");
        assert!(result.items().len() >= 2);
        assert!(result.metadata().total_count() >= 2);
    }

    #[tokio::test]
    async fn test_list_all_default_pagination() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username = UserName::new("list_all_default_user").expect("Invalid username");
        let user = User::create(username, "hash".to_string());
        repo.create(&user).await.expect("Failed to create");

        let result = repo
            .list_all(None)
            .await
            .expect("Failed to list all with default pagination");
        assert!(!result.items().is_empty());
    }

    #[tokio::test]
    async fn test_username_exists() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username = UserName::new("exists_user").expect("Invalid username");
        let user = User::create(username.clone(), "hash".to_string());
        repo.create(&user).await.expect("Failed to create");

        let exists = repo
            .username_exists(&username)
            .await
            .expect("Failed to check username_exists");
        assert!(exists);
    }

    #[tokio::test]
    async fn test_username_exists_false() {
        let db = setup().await;
        let repo = SurrealUserRepository::new(db);

        let username = UserName::new("does_not_exist_user").expect("Invalid username");
        let exists = repo
            .username_exists(&username)
            .await
            .expect("Failed to check username_exists");
        assert!(!exists);
    }
}
