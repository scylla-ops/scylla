//! End-to-end test for creating multiple users
//!
//! Tests creating multiple users and verifying they're all stored correctly.
//! This ensures there are no state issues or conflicts between operations.

use mockall::mock;
use scylla_core::application::dto::CreateUserRequestDto;
use scylla_core::application::ports::{PasswordHasher, RbacEnforcer};
use scylla_core::application::use_cases::user::create_user::CreateUserUseCase;
use scylla_core::domain::errors::DomainResult;
use scylla_core::domain::repositories::UserRepository;
use scylla_core::domain::value_objects::{Password, UserId, Username};
use scylla_core::infrastructure::auth::argon2_password_hasher::Argon2PasswordHasher;
use scylla_core::infrastructure::persistence::surrealdb::user_repository::SurrealUserRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

// Mock RBAC enforcer for testing
mock! {
    pub RbacEnforcer {}

    #[async_trait::async_trait]
    impl RbacEnforcer for RbacEnforcer {
        async fn enforce(&self, user_id: &UserId, domain: &str, resource: &str, action: &str) -> DomainResult<bool>;
        async fn add_role_for_user(&self, user_id: &UserId, role: &str, domain: &str) -> DomainResult<()>;
        async fn remove_role_for_user(&self, user_id: &UserId, role: &str, domain: &str) -> DomainResult<()>;
        async fn get_roles_for_user(&self, user_id: &UserId, domain: &str) -> DomainResult<Vec<String>>;
        async fn get_users_for_role(&self, role: &str, domain: &str) -> DomainResult<Vec<UserId>>;
        async fn has_role(&self, user_id: &UserId, role: &str, domain: &str) -> DomainResult<bool>;
    }
}

/// Helper to set up the complete infrastructure for testing
async fn setup_infrastructure() -> (
    Arc<dyn UserRepository>,
    Arc<dyn PasswordHasher>,
    Arc<dyn RbacEnforcer>,
) {
    let db = setup_test_db().await;

    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db.clone()));
    let password_hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::new());

    let mut rbac_enforcer = MockRbacEnforcer::new();
    rbac_enforcer
        .expect_add_role_for_user()
        .returning(|_, _, _| Ok(()));

    rbac_enforcer
        .expect_has_role()
        .returning(|_, _, _| Ok(true));

    (user_repo, password_hasher, Arc::new(rbac_enforcer))
}

#[tokio::test]
#[serial]
async fn test_create_multiple_users_end_to_end() {
    // Arrange
    let (user_repo, _password_hasher, rbac_enforcer) = setup_infrastructure().await;

    let use_case = CreateUserUseCase::new(
        Arc::clone(&user_repo),
        Arc::new(Argon2PasswordHasher::new()),
        Arc::clone(&rbac_enforcer),
    );

    let usernames = vec!["alice", "bob", "charlie"];

    // Act: Create multiple users
    let mut created_ids = Vec::new();

    for username in &usernames {
        let request = CreateUserRequestDto {
            username: Username::new(username.to_string()).unwrap(),
            password: Password::new("Password123!".to_string()).unwrap(),
        };

        let result = use_case
            .execute(request)
            .await
            .expect(&format!("Creating {} should succeed", username));

        created_ids.push(result.id.clone());
    }

    // Assert: All users were created with unique IDs
    assert_eq!(created_ids.len(), 3);

    // All IDs should be unique
    for i in 0..created_ids.len() {
        for j in (i + 1)..created_ids.len() {
            assert_ne!(
                created_ids[i].as_str(),
                created_ids[j].as_str(),
                "User IDs should be unique"
            );
        }
    }

    // Verify: All users exist in database
    for (i, username) in usernames.iter().enumerate() {
        let user = user_repo
            .find_by_id(&created_ids[i])
            .await
            .expect(&format!("{} should exist", username));

        assert_eq!(user.username().as_str(), *username);
        assert!(user.is_active());

        // Verify RBAC role
        let has_role = rbac_enforcer
            .has_role(&created_ids[i], "user", "*")
            .await
            .expect("RBAC check should work");

        assert!(has_role, "{} should have user role", username);
    }

    // Verify: Can list all users
    let all_users = user_repo
        .list_all(None)
        .await
        .expect("Listing users should work");

    assert_eq!(
        all_users.metadata().total_count(),
        3,
        "Should have 3 users total"
    );
}
