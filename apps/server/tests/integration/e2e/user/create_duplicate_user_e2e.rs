//! End-to-end test for duplicate user creation
//!
//! Verifies that duplicate username handling works correctly
//! across the entire stack.

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

    (user_repo, password_hasher, Arc::new(rbac_enforcer))
}

#[tokio::test]
#[serial]
async fn test_create_duplicate_user_end_to_end() {
    // Arrange
    let (user_repo, password_hasher, rbac_enforcer) = setup_infrastructure().await;

    let use_case = CreateUserUseCase::new(
        Arc::clone(&user_repo),
        Arc::clone(&password_hasher),
        Arc::clone(&rbac_enforcer),
    );

    let request1 = CreateUserRequestDto {
        username: Username::new("duplicate".to_string()).unwrap(),
        password: Password::new("Password123!".to_string()).unwrap(),
    };

    // Act: Create first user
    let result1 = use_case.execute(request1).await;
    assert!(result1.is_ok(), "First user creation should succeed");

    // Act: Try to create duplicate user
    let request2 = CreateUserRequestDto {
        username: Username::new("duplicate".to_string()).unwrap(),
        password: Password::new("DifferentPass123!".to_string()).unwrap(),
    };

    let result2 = use_case.execute(request2).await;

    // Assert: Second creation should fail with conflict
    assert!(
        result2.is_err(),
        "Creating duplicate user should fail end-to-end"
    );

    // Verify: Only one user exists in database
    let username = Username::new("duplicate".to_string()).unwrap();
    let exists = user_repo
        .username_exists(&username)
        .await
        .expect("Username check should work");

    assert!(exists, "First user should still exist");

    // Verify: We can still find the first user (it wasn't corrupted)
    let stored_user = user_repo
        .find_by_username(&username)
        .await
        .expect("First user should be retrievable");

    // The stored password should match the first user's password, not the second
    let first_password = Password::new("Password123!".to_string()).unwrap();
    let is_valid = password_hasher
        .verify(&first_password, stored_user.password_hash())
        .await
        .expect("Verification should work");

    assert!(is_valid, "First user's password should still be correct");
}
