//! Integration tests for ListUsersUseCase

use scylla_core::application::dto::ListUsersRequestDto;
use scylla_core::application::use_cases::user::list_users::ListUsersUseCase;
use scylla_core::domain::entities::User;
use scylla_core::domain::repositories::UserRepository;
use scylla_core::domain::value_objects::{PaginationParams, Username};
use scylla_core::infrastructure::persistence::surrealdb::user_repository::SurrealUserRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

fn create_test_user(username: &str) -> User {
    User::create(
        Username::new(username.to_string()).unwrap(),
        "hashed_password".to_string(),
    )
}

#[tokio::test]
#[serial]
async fn test_list_users_use_case_success() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db.clone()));
    let use_case = ListUsersUseCase::new(user_repo.clone());

    // Create multiple users
    for i in 0..5 {
        let user = create_test_user(&format!("user{}", i));
        user_repo.create(&user).await.unwrap();
    }

    let request = ListUsersRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List users should succeed");
    let response = result.unwrap();
    assert_eq!(response.users.len(), 5);
    assert!(response.pagination.is_some());
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 5);
    }
}

#[tokio::test]
#[serial]
async fn test_list_users_use_case_with_pagination() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db.clone()));
    let use_case = ListUsersUseCase::new(user_repo.clone());

    // Create 10 users
    for i in 0..10 {
        let user = create_test_user(&format!("paginated{}", i));
        user_repo.create(&user).await.unwrap();
    }

    let pagination = PaginationParams::new(1, 3).unwrap();
    let request = ListUsersRequestDto {
        pagination: Some(pagination),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List users with pagination should succeed");
    let response = result.unwrap();
    assert_eq!(response.users.len(), 3);
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 10);
    }
}

#[tokio::test]
#[serial]
async fn test_list_users_use_case_empty() {
    let db = setup_test_db().await;
    let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db));
    let use_case = ListUsersUseCase::new(user_repo);

    let request = ListUsersRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List users should succeed even when empty");
    let response = result.unwrap();
    assert_eq!(response.users.len(), 0);
}
