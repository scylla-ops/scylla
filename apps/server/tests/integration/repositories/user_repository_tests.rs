//! Integration tests for UserRepository
//!
//! These tests verify the UserRepository implementation against a real SurrealDB in-memory database.
//! Unlike unit tests that use mocks, these tests ensure that:
//! - SQL queries are correct
//! - Database constraints work as expected
//! - Data is properly persisted and retrieved
//! - Error cases are handled correctly
//!
//! # Test Pattern: Repository Integration Tests
//!
//! Each test:
//! 1. Creates a fresh in-memory database
//! 2. Performs repository operations
//! 3. Verifies results against the database
//! 4. Tests are isolated (no shared state)

use scylla_core::domain::entities::User;
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::repositories::UserRepository;
use scylla_core::domain::value_objects::{PaginationParams, Username};
use scylla_core::infrastructure::persistence::surrealdb::user_repository::SurrealUserRepository;
use serial_test::serial;

use crate::common::setup_test_db;

/// Helper to create a test user entity
fn create_test_user(username: &str) -> User {
    User::create(
        Username::new(username.to_string()).unwrap(),
        "hashed_password".to_string(),
    )
}

// ===== CREATE Tests =====

/// Test Pattern: Basic CRUD - Create
///
/// Verifies that a user can be successfully created in the database
/// and all fields are persisted correctly.
#[tokio::test]
#[serial] // Serialize tests to avoid database conflicts
async fn test_create_user_success() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let user = create_test_user("testuser");

    // Act
    let created_user = repo
        .create(&user)
        .await
        .expect("Creating user should succeed");

    // Assert
    assert_eq!(created_user.username().as_str(), "testuser");
    assert_eq!(created_user.password_hash(), "hashed_password");
    assert!(created_user.is_active(), "New user should be active");

    // Verify the user can be retrieved using the created user's ID
    let retrieved_user = repo
        .find_by_id(created_user.id())
        .await
        .expect("Should be able to retrieve created user");

    assert_eq!(retrieved_user.username().as_str(), "testuser");
}

/// Test Pattern: Constraint Validation
///
/// Verifies that database constraints prevent duplicate usernames.
/// This is critical for data integrity.
#[tokio::test]
#[serial]
async fn test_create_user_with_duplicate_username_fails() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let user1 = create_test_user("duplicate");
    let user2 = create_test_user("duplicate"); // Same username

    // Act: Create first user
    repo.create(&user1)
        .await
        .expect("First user creation should succeed");

    // Act: Try to create second user with same username
    let result = repo.create(&user2).await;

    // Assert: Should fail due to unique constraint
    assert!(
        result.is_err(),
        "Creating user with duplicate username should fail"
    );

    // The error should indicate a conflict/constraint violation
    match result.unwrap_err() {
        DomainError::Conflict(_) | DomainError::Internal(_) | DomainError::Infrastructure(_) => {
            // Error types that are acceptable for constraint violations
        }
        other => panic!(
            "Expected Conflict, Internal, or Infrastructure error, got {:?}",
            other
        ),
    }
}

// ===== FIND_BY_ID Tests =====

/// Test Pattern: Basic CRUD - Read by ID
///
/// Verifies that a user can be retrieved by their ID.
#[tokio::test]
#[serial]
async fn test_find_user_by_id_success() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let user = create_test_user("findme");
    let created_user = repo.create(&user).await.unwrap();
    let user_id = created_user.id().clone();

    // Act
    let found_user = repo
        .find_by_id(&user_id)
        .await
        .expect("Finding user by ID should succeed");

    // Assert
    assert_eq!(found_user.id().as_str(), user_id.as_str());
    assert_eq!(found_user.username().as_str(), "findme");
}

/// Test Pattern: Not Found Error
///
/// Verifies that querying for a non-existent user returns an appropriate error.
#[tokio::test]
#[serial]
async fn test_find_user_by_id_not_found() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let fake_id = scylla_core::domain::value_objects::UserId::new("users:nonexistent");

    // Act
    let result = repo.find_by_id(&fake_id).await;

    // Assert
    assert!(result.is_err(), "Finding non-existent user should fail");

    match result.unwrap_err() {
        DomainError::NotFound {
            entity_type: _,
            id: _,
        } => {
            // Expected error type
        }
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ===== FIND_BY_USERNAME Tests =====

/// Test Pattern: Query by Unique Field
///
/// Verifies that users can be found by their username.
#[tokio::test]
#[serial]
async fn test_find_user_by_username_success() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let user = create_test_user("searchable");
    repo.create(&user).await.unwrap();

    let username = Username::new("searchable".to_string()).unwrap();

    // Act
    let found_user = repo
        .find_by_username(&username)
        .await
        .expect("Finding user by username should succeed");

    // Assert
    assert_eq!(found_user.username().as_str(), "searchable");
}

#[tokio::test]
#[serial]
async fn test_find_user_by_username_not_found() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let username = Username::new("nonexistent".to_string()).unwrap();

    // Act
    let result = repo.find_by_username(&username).await;

    // Assert
    assert!(result.is_err(), "Finding non-existent username should fail");

    match result.unwrap_err() {
        DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ===== UPDATE Tests =====

/// Test Pattern: Basic CRUD - Update
///
/// Verifies that user data can be updated and persisted correctly.
#[tokio::test]
#[serial]
async fn test_update_user_success() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let user = create_test_user("updateme");
    let mut created_user = repo.create(&user).await.unwrap();

    // Modify the user
    let new_username = Username::new("updated".to_string()).unwrap();
    created_user.update_username(new_username).unwrap();

    // Act
    let updated_user = repo
        .update(&created_user)
        .await
        .expect("Updating user should succeed");

    // Assert
    assert_eq!(updated_user.username().as_str(), "updated");

    // Verify the update persisted
    let retrieved_user = repo.find_by_id(created_user.id()).await.unwrap();
    assert_eq!(retrieved_user.username().as_str(), "updated");
}

/// Test Pattern: Update State Transitions
///
/// Verifies that user activation/deactivation is properly persisted.
#[tokio::test]
#[serial]
async fn test_update_user_deactivation_persists() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let user = create_test_user("deactivateme");
    let mut created_user = repo.create(&user).await.unwrap();

    // User starts active
    assert!(created_user.is_active());

    // Deactivate
    created_user.deactivate().unwrap();

    // Act: Update in database
    let updated_user = repo.update(&created_user).await.unwrap();

    // Assert: Deactivation persisted
    assert!(!updated_user.is_active());

    // Verify by re-fetching
    let retrieved_user = repo.find_by_id(created_user.id()).await.unwrap();
    assert!(!retrieved_user.is_active());
}

// ===== DELETE Tests =====

/// Test Pattern: Basic CRUD - Delete
///
/// Verifies that users can be deleted from the database.
#[tokio::test]
#[serial]
async fn test_delete_user_success() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let user = create_test_user("deleteme");
    let created_user = repo.create(&user).await.unwrap();
    let user_id = created_user.id().clone();

    // Verify user exists
    assert!(repo.find_by_id(&user_id).await.is_ok());

    // Act
    repo.delete(&user_id)
        .await
        .expect("Deleting user should succeed");

    // Assert: User no longer exists
    let result = repo.find_by_id(&user_id).await;
    assert!(result.is_err(), "Deleted user should not be found");

    match result.unwrap_err() {
        DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn test_delete_nonexistent_user() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let fake_id = scylla_core::domain::value_objects::UserId::new("users:nonexistent");

    // Act
    let result = repo.delete(&fake_id).await;

    // Assert: Deleting non-existent user might succeed (idempotent) or fail
    // Either behavior is acceptable, but document which one your repo implements
    // For this example, we'll accept either
    match result {
        Ok(_) => {
            // Idempotent delete - acceptable
        }
        Err(DomainError::NotFound { .. }) => {
            // Explicit error - also acceptable
        }
        Err(other) => panic!("Unexpected error type: {:?}", other),
    }
}

// ===== LIST_ALL Tests =====

/// Test Pattern: List/Query Operations
///
/// Verifies that multiple users can be retrieved and pagination works.
#[tokio::test]
#[serial]
async fn test_list_all_users() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    // Create multiple users
    for i in 1..=5 {
        let user = create_test_user(&format!("user{}", i));
        repo.create(&user).await.unwrap();
    }

    // Act: List all users without pagination
    let result = repo
        .list_all(None)
        .await
        .expect("Listing all users should succeed");

    // Assert
    assert_eq!(result.items().len(), 5, "Should have 5 users");
    assert_eq!(
        result.metadata().total_count(),
        5,
        "Total count should be 5"
    );
}

/// Test Pattern: Pagination
///
/// Verifies that pagination limits and offsets work correctly.
#[tokio::test]
#[serial]
async fn test_list_users_with_pagination() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    // Create 10 users
    for i in 1..=10 {
        let user = create_test_user(&format!("paginated{}", i));
        repo.create(&user).await.unwrap();
    }

    // Act: Get first page (page 1, 5 items per page)
    let pagination = PaginationParams::new(1, 5).unwrap();
    let page1 = repo
        .list_all(Some(&pagination))
        .await
        .expect("Listing first page should succeed");

    // Assert: First page
    assert_eq!(page1.items().len(), 5, "First page should have 5 items");
    assert_eq!(page1.metadata().total_count(), 10, "Total should be 10");

    // Act: Get second page (page 2, 5 items per page)
    let pagination2 = PaginationParams::new(2, 5).unwrap();
    let page2 = repo
        .list_all(Some(&pagination2))
        .await
        .expect("Listing second page should succeed");

    // Assert: Second page
    assert_eq!(page2.items().len(), 5, "Second page should have 5 items");
    assert_eq!(
        page2.metadata().total_count(),
        10,
        "Total should still be 10"
    );

    // Verify no duplicates between pages
    let page1_ids: Vec<_> = page1.items().iter().map(|u| u.id().as_str()).collect();
    let page2_ids: Vec<_> = page2.items().iter().map(|u| u.id().as_str()).collect();

    for id in &page1_ids {
        assert!(
            !page2_ids.contains(id),
            "Pages should not have duplicate users"
        );
    }
}

// ===== USERNAME_EXISTS Tests =====

/// Test Pattern: Existence Checks
///
/// Verifies that username existence checks work correctly.
/// This is critical for the CreateUser use case.
#[tokio::test]
#[serial]
async fn test_username_exists_returns_true_when_exists() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let user = create_test_user("existing");
    repo.create(&user).await.unwrap();

    let username = Username::new("existing".to_string()).unwrap();

    // Act
    let exists = repo
        .username_exists(&username)
        .await
        .expect("Checking username existence should succeed");

    // Assert
    assert!(exists, "Username should exist");
}

#[tokio::test]
#[serial]
async fn test_username_exists_returns_false_when_not_exists() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    let username = Username::new("notexisting".to_string()).unwrap();

    // Act
    let exists = repo
        .username_exists(&username)
        .await
        .expect("Checking username existence should succeed");

    // Assert
    assert!(!exists, "Username should not exist");
}

// ===== Complex Scenarios =====

/// Test Pattern: Complete Workflow
///
/// Tests a complete user lifecycle: create, read, update, delete.
#[tokio::test]
#[serial]
async fn test_user_full_lifecycle() {
    // Arrange
    let db = setup_test_db().await;
    let repo = SurrealUserRepository::new(db);

    // 1. Create
    let user = create_test_user("lifecycle");
    let created = repo.create(&user).await.expect("Create should succeed");
    assert!(created.is_active());

    // 2. Read
    let found = repo
        .find_by_id(created.id())
        .await
        .expect("Find should succeed");
    assert_eq!(found.username().as_str(), "lifecycle");

    // 3. Update
    let mut updated = found;
    updated.deactivate().expect("Deactivate should succeed");
    let persisted = repo.update(&updated).await.expect("Update should succeed");
    assert!(!persisted.is_active());

    // 4. Delete
    repo.delete(persisted.id())
        .await
        .expect("Delete should succeed");

    // 5. Verify deletion
    let result = repo.find_by_id(persisted.id()).await;
    assert!(result.is_err(), "User should be deleted");
}
