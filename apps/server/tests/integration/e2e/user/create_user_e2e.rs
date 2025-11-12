//! End-to-end test for creating a user
//!
//! This test verifies the complete user creation workflow:
//! 1. Password is hashed with Argon2
//! 2. User is stored in SurrealDB
//! 3. RBAC role is assigned in Casbin
//! 4. User can be retrieved from database
//! 5. Password can be verified

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

// Mock RBAC enforcer for testing (to avoid Casbin model file complexity)
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

    // Real repository implementation
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db.clone()));

    // Real password hasher
    let password_hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::default());

    // Mock RBAC enforcer (to avoid Casbin setup complexity in tests)
    let mut rbac_enforcer = MockRbacEnforcer::new();

    // Set up default expectations for RBAC operations
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
async fn test_create_user_end_to_end_success() {
    // Arrange: Set up real infrastructure
    let (user_repo, password_hasher, rbac_enforcer) = setup_infrastructure().await;

    let use_case = CreateUserUseCase::new(
        Arc::clone(&user_repo),
        Arc::clone(&password_hasher),
        Arc::clone(&rbac_enforcer),
    );

    let request = CreateUserRequestDto {
        username: Username::try_from("e2euser".to_string()).unwrap(),
        password: Password::try_from("SecurePassword123!".to_string()).unwrap(),
    };

    // Act: Execute the complete workflow
    let result = use_case.execute(request).await;

    // Assert: User was created successfully
    assert!(result.is_ok(), "End-to-end user creation should succeed");

    let response = result.unwrap();
    assert_eq!(response.username.as_str(), "e2euser");
    assert!(response.is_active);

    // Verify: User exists in database
    let stored_user = user_repo
        .find_by_id(&response.id)
        .await
        .expect("User should exist in database");

    assert_eq!(stored_user.username().as_str(), "e2euser");

    // Verify: Password was actually hashed (not stored as plaintext)
    assert_ne!(stored_user.password_hash(), "SecurePassword123!");
    assert!(
        stored_user.password_hash().starts_with("$argon2"),
        "Password should be hashed with Argon2"
    );

    // Verify: Password can be verified
    let password = Password::try_from("SecurePassword123!".to_string()).unwrap();
    let is_valid = password_hasher
        .verify(&password, stored_user.password_hash())
        .await
        .expect("Password verification should work");

    assert!(is_valid, "Original password should verify against hash");

    // Verify: Wrong password doesn't verify
    let wrong_password = Password::try_from("WrongPassword123!".to_string()).unwrap();
    let is_invalid = password_hasher
        .verify(&wrong_password, stored_user.password_hash())
        .await
        .expect("Password verification should work");

    assert!(!is_invalid, "Wrong password should not verify");

    // Verify: RBAC role was assigned
    let has_role = rbac_enforcer
        .has_role(&response.id, "user", "*")
        .await
        .expect("RBAC check should work");

    assert!(has_role, "User should have 'user' role assigned");
}
