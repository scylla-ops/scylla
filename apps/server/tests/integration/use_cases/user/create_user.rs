//! Integration tests for CreateUserUseCase
//!
//! These tests verify the CreateUserUseCase with real repositories and services.
//! Unlike unit tests that use mocks, these tests use real implementations.

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

#[tokio::test]
#[serial]
async fn test_create_user_use_case_with_real_repository() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let password_hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::default());

    let mut rbac_enforcer = MockRbacEnforcer::new();
    rbac_enforcer
        .expect_add_role_for_user()
        .returning(|_, _, _| Ok(()));

    let use_case = CreateUserUseCase::new(
        user_repo.clone(),
        password_hasher.clone(),
        Arc::new(rbac_enforcer),
    );

    let request = CreateUserRequestDto {
        username: Username::try_from("usecasetest".to_string()).unwrap(),
        password: Password::try_from("SecurePass123!".to_string()).unwrap(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Use case should succeed");
    let response = result.unwrap();
    assert_eq!(response.username.as_str(), "usecasetest");
    assert!(response.is_active);

    // Verify user exists in repository
    let user = user_repo
        .find_by_id(&response.id)
        .await
        .expect("User should exist");
    assert_eq!(user.username().as_str(), "usecasetest");
}

#[tokio::test]
#[serial]
async fn test_create_user_use_case_duplicate_username() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let password_hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::default());

    let mut rbac_enforcer = MockRbacEnforcer::new();
    // Expect only one call to add_role_for_user (for the first user)
    rbac_enforcer
        .expect_add_role_for_user()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let use_case =
        CreateUserUseCase::new(user_repo.clone(), password_hasher, Arc::new(rbac_enforcer));

    let request1 = CreateUserRequestDto {
        username: Username::try_from("duplicate".to_string()).unwrap(),
        password: Password::try_from("Pass123!".to_string()).unwrap(),
    };

    let result1 = use_case.execute(request1).await;
    assert!(result1.is_ok(), "First creation should succeed");

    let rbac_enforcer2 = MockRbacEnforcer::new();
    // For duplicate test, RBAC should not be called (fails at username check)
    let use_case2 = CreateUserUseCase::new(
        user_repo.clone(),
        Arc::new(Argon2PasswordHasher::default()),
        Arc::new(rbac_enforcer2),
    );

    let request2 = CreateUserRequestDto {
        username: Username::try_from("duplicate".to_string()).unwrap(),
        password: Password::try_from("Pass456!".to_string()).unwrap(),
    };

    let result2 = use_case2.execute(request2).await;
    assert!(result2.is_err(), "Duplicate username should fail");
}
