//! Integration tests for DeleteUserUseCase

use scylla_core::application::dto::DeleteUserRequestDto;
use scylla_core::application::use_cases::user::delete_user::DeleteUserUseCase;
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
async fn test_delete_user_use_case_success() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db.clone()));
    let use_case = DeleteUserUseCase::new(user_repo.clone());

    let user = create_test_user("todelete");
    let created = user_repo.create(&user).await.unwrap();

    let request = DeleteUserRequestDto {
        user_id: created.id().clone(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Delete user should succeed");

    // Verify user is deleted
    let get_result = user_repo.find_by_id(created.id()).await;
    assert!(get_result.is_err(), "User should be deleted");
}

#[tokio::test]
#[serial]
async fn test_delete_user_use_case_not_found() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let use_case = DeleteUserUseCase::new(user_repo);

    let request = DeleteUserRequestDto {
        user_id: scylla_core::domain::value_objects::UserId::generate(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Delete non-existent user should fail");
    match result.unwrap_err() {
        scylla_core::domain::errors::DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}
