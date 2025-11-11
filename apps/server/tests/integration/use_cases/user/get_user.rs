//! Integration tests for GetUserUseCase

use scylla_core::application::dto::GetUserRequestDto;
use scylla_core::application::use_cases::user::get_user::GetUserUseCase;
use scylla_core::domain::entities::User;
use scylla_core::domain::repositories::UserRepository;
use scylla_core::domain::value_objects::Username;
use scylla_core::infrastructure::persistence::surrealdb::user_repository::SurrealUserRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

fn create_test_user(username: &str) -> User {
    User::create(
        Username::try_from(username.to_string()).unwrap(),
        "hashed_password".to_string(),
    )
}

#[tokio::test]
#[serial]
async fn test_get_user_use_case_success() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let use_case = GetUserUseCase::new(user_repo.clone());

    let user = create_test_user("getuser");
    let created = user_repo.create(&user).await.unwrap();

    let request = GetUserRequestDto {
        user_id: created.id().clone(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Get user should succeed");
    let response = result.unwrap();
    assert_eq!(response.username.as_str(), "getuser");
    assert!(response.is_active);
}

#[tokio::test]
#[serial]
async fn test_get_user_use_case_not_found() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let use_case = GetUserUseCase::new(user_repo);

    let request = GetUserRequestDto {
        user_id: scylla_core::domain::value_objects::UserId::generate(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Get non-existent user should fail");
    match result.unwrap_err() {
        scylla_core::domain::errors::DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}
