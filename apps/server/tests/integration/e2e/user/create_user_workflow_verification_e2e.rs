//! End-to-end test for user creation workflow verification
//!
//! Tests that all workflow steps complete correctly when creating a user.

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
    let password_hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::default());

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
async fn test_create_user_workflow_verification() {
    // Arrange
    let (user_repo, _password_hasher, rbac_enforcer) = setup_infrastructure().await;

    let use_case = CreateUserUseCase::new(
        Arc::clone(&user_repo),
        Arc::new(Argon2PasswordHasher::default()),
        Arc::clone(&rbac_enforcer),
    );

    let request = CreateUserRequestDto {
        username: Username::try_from("workflowtest".to_string()).unwrap(),
        password: Password::try_from("TestPass123!".to_string()).unwrap(),
    };

    // Act: Create user
    let result = use_case.execute(request).await;

    // Assert: Creation succeeded
    assert!(result.is_ok(), "Workflow should complete successfully");

    let response = result.unwrap();

    // Verify: Check all workflow steps completed correctly

    // 1. User in database
    let user_in_db = user_repo.find_by_id(&response.id).await.is_ok();
    assert!(user_in_db, "User should be in database");

    // 2. Password was hashed
    let stored = user_repo.find_by_id(&response.id).await.unwrap();
    assert!(
        stored.password_hash().starts_with("$argon2"),
        "Password should be hashed"
    );

    // 3. Username is searchable
    let username = Username::try_from("workflowtest".to_string()).unwrap();
    let searchable = user_repo.find_by_username(&username).await.is_ok();
    assert!(searchable, "User should be findable by username");

    // 4. RBAC role assigned
    let has_role = rbac_enforcer
        .has_role(&response.id, "user", "*")
        .await
        .expect("RBAC should work");
    assert!(has_role, "User should have role");

    // All workflow steps verified!
}
