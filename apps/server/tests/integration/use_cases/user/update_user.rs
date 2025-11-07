//! Integration tests for UpdateUserUseCase

use scylla_core::application::dto::UpdateUserRequestDto;
use scylla_core::application::use_cases::user::update_user::UpdateUserUseCase;
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
async fn test_update_user_use_case_update_username() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let use_case = UpdateUserUseCase::new(user_repo.clone());

    let user = create_test_user("original");
    let created = user_repo.create(&user).await.unwrap();

    let request = UpdateUserRequestDto {
        user_id: created.id().clone(),
        username: Some(Username::try_from("updated_username".to_string()).unwrap()),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Update user should succeed");
    let response = result.unwrap();
    assert_eq!(response.username.as_str(), "updated_username");

    // Verify it's persisted
    let updated = user_repo.find_by_id(&response.id).await.unwrap();
    assert_eq!(updated.username().as_str(), "updated_username");
}

#[tokio::test]
#[serial]
async fn test_update_user_use_case_duplicate_username() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let use_case = UpdateUserUseCase::new(user_repo.clone());

    let user1 = create_test_user("user1");
    let user2 = create_test_user("user2");
    let _created1 = user_repo.create(&user1).await.unwrap();
    let created2 = user_repo.create(&user2).await.unwrap();

    // Try to update user2's username to user1's username
    let request = UpdateUserRequestDto {
        user_id: created2.id().clone(),
        username: Some(Username::try_from("user1".to_string()).unwrap()),
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_err(),
        "Update with duplicate username should fail"
    );
    match result.unwrap_err() {
        scylla_core::domain::errors::DomainError::Conflict(_) => {}
        other => panic!("Expected Conflict error, got {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn test_update_user_use_case_same_username() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let use_case = UpdateUserUseCase::new(user_repo.clone());

    let user = create_test_user("sameuser");
    let created = user_repo.create(&user).await.unwrap();

    // Update to the same username should succeed
    let request = UpdateUserRequestDto {
        user_id: created.id().clone(),
        username: Some(Username::try_from("sameuser".to_string()).unwrap()),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Update to same username should succeed");
}

#[tokio::test]
#[serial]
async fn test_update_user_use_case_not_found() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let use_case = UpdateUserUseCase::new(user_repo);

    let request = UpdateUserRequestDto {
        user_id: scylla_core::domain::value_objects::UserId::generate(),
        username: Some(Username::try_from("newuser".to_string()).unwrap()),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Update non-existent user should fail");
}
