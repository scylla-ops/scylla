//! Integration tests for OrganizationRepository
//!
//! These tests verify the OrganizationRepository implementation against a real SurrealDB in-memory database.

use scylla_core::domain::entities::Organization;
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::repositories::OrganizationRepository;
use scylla_core::domain::value_objects::{Description, OrganizationName};
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use serial_test::serial;

use crate::common::setup_test_db;

/// Helper to create a test organization entity
fn create_test_organization(name: &str, description: Option<&str>) -> Organization {
    Organization::create(
        OrganizationName::new(name.to_string()).unwrap(),
        description.map(|d| Description::new(d.to_string()).unwrap()),
    )
    .unwrap()
}

// ===== CREATE Tests =====

#[tokio::test]
#[serial]
async fn test_create_organization_success() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org = create_test_organization("TestOrg", Some("Test description"));

    let created = repo
        .create(&org)
        .await
        .expect("Creating organization should succeed");

    assert_eq!(created.name().as_str(), "TestOrg");
    assert!(created.is_active());
    assert_eq!(
        created.description().as_ref().map(|d| d.as_str()),
        Some("Test description")
    );

    let retrieved = repo
        .find_by_id(created.id())
        .await
        .expect("Should be able to retrieve created organization");

    assert_eq!(retrieved.name().as_str(), "TestOrg");
}

#[tokio::test]
#[serial]
async fn test_create_organization_with_duplicate_name_fails() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org1 = create_test_organization("Duplicate", None);
    let org2 = create_test_organization("Duplicate", None);

    repo.create(&org1)
        .await
        .expect("First organization creation should succeed");

    let result = repo.create(&org2).await;

    assert!(
        result.is_err(),
        "Creating organization with duplicate name should fail"
    );

    match result.unwrap_err() {
        DomainError::Conflict(_) | DomainError::Internal(_) | DomainError::Infrastructure(_) => {}
        other => panic!(
            "Expected Conflict, Internal, or Infrastructure error, got {:?}",
            other
        ),
    }
}

// ===== FIND_BY_ID Tests =====

#[tokio::test]
#[serial]
async fn test_find_by_id_success() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org = create_test_organization("FindMe", None);
    let created = repo.create(&org).await.unwrap();

    let found = repo
        .find_by_id(created.id())
        .await
        .expect("Should find organization by ID");

    assert_eq!(found.name().as_str(), "FindMe");
}

#[tokio::test]
#[serial]
async fn test_find_by_id_not_found() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org_id = scylla_core::domain::value_objects::OrganizationId::generate();

    let result = repo.find_by_id(&org_id).await;

    assert!(result.is_err(), "Should return error when not found");
    match result.unwrap_err() {
        DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ===== FIND_BY_NAME Tests =====

#[tokio::test]
#[serial]
async fn test_find_by_name_success() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org = create_test_organization("FindByName", None);
    repo.create(&org).await.unwrap();

    let name = OrganizationName::new("FindByName".to_string()).unwrap();
    let found = repo
        .find_by_name(&name)
        .await
        .expect("Should find organization by name");

    assert_eq!(found.name().as_str(), "FindByName");
}

#[tokio::test]
#[serial]
async fn test_find_by_name_not_found() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let name = OrganizationName::new("NonExistent".to_string()).unwrap();

    let result = repo.find_by_name(&name).await;

    assert!(result.is_err(), "Should return error when not found");
}

// ===== UPDATE Tests =====

#[tokio::test]
#[serial]
async fn test_update_organization_success() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org = create_test_organization("Original", None);
    let created = repo.create(&org).await.unwrap();

    let mut updated = created;
    let new_name = OrganizationName::new("Updated".to_string()).unwrap();
    updated.update_name(new_name).unwrap();
    updated
        .update_description(Some(
            Description::new("New description".to_string()).unwrap(),
        ))
        .unwrap();

    let persisted = repo.update(&updated).await.expect("Update should succeed");

    assert_eq!(persisted.name().as_str(), "Updated");
    assert_eq!(
        persisted.description().as_ref().map(|d| d.as_str()),
        Some("New description")
    );
}

// ===== DELETE Tests =====

#[tokio::test]
#[serial]
async fn test_delete_organization_success() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org = create_test_organization("DeleteMe", None);
    let created = repo.create(&org).await.unwrap();

    repo.delete(created.id())
        .await
        .expect("Delete should succeed");

    let result = repo.find_by_id(created.id()).await;
    assert!(result.is_err(), "Organization should be deleted");
}

#[tokio::test]
#[serial]
async fn test_delete_nonexistent_organization() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org_id = scylla_core::domain::value_objects::OrganizationId::generate();

    let result = repo.delete(&org_id).await;
    assert!(
        result.is_ok(),
        "Deleting non-existent organization should not error"
    );
}

// ===== LIST_ALL Tests =====

#[tokio::test]
#[serial]
async fn test_list_all_organizations() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    for i in 0..5 {
        let org = create_test_organization(&format!("Org{}", i), None);
        repo.create(&org).await.unwrap();
    }

    let result = repo.list_all(None).await.expect("Listing should succeed");

    assert_eq!(result.items().len(), 5);
    assert_eq!(result.metadata().total_count(), 5);
}

// ===== LIST_ACTIVE Tests =====

#[tokio::test]
#[serial]
async fn test_list_active_organizations() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    for i in 0..3 {
        let org = create_test_organization(&format!("ActiveOrg{}", i), None);
        let created = repo.create(&org).await.unwrap();
        if i == 1 {
            let mut org_draft = repo.find_by_id(&created.id()).await.unwrap();
            org_draft.toggle_active().unwrap();
            repo.update(&org_draft).await.unwrap();
        }
    }

    let result = repo
        .list_active(None)
        .await
        .expect("Listing active should succeed");

    assert_eq!(
        result.items().len(),
        2,
        "Should have 2 active organizations"
    );
}

// ===== NAME_EXISTS Tests =====

#[tokio::test]
#[serial]
async fn test_name_exists_returns_true_when_exists() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org = create_test_organization("Existing", None);
    repo.create(&org).await.unwrap();

    let name = OrganizationName::new("Existing".to_string()).unwrap();

    let exists = repo
        .name_exists(&name)
        .await
        .expect("Checking name existence should succeed");

    assert!(exists, "Name should exist");
}

#[tokio::test]
#[serial]
async fn test_name_exists_returns_false_when_not_exists() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let name = OrganizationName::new("NonExistent".to_string()).unwrap();

    let exists = repo
        .name_exists(&name)
        .await
        .expect("Checking name existence should succeed");

    assert!(!exists, "Name should not exist");
}

// ===== Complex Scenarios =====

#[tokio::test]
#[serial]
async fn test_organization_full_lifecycle() {
    let db = setup_test_db().await;
    let repo = SurrealOrganizationRepository::new(db);

    let org = create_test_organization("Lifecycle", Some("Initial"));
    let created = repo.create(&org).await.expect("Create should succeed");
    assert!(created.is_active());

    let found = repo
        .find_by_id(created.id())
        .await
        .expect("Find should succeed");
    assert_eq!(found.name().as_str(), "Lifecycle");

    let mut updated = found;
    updated
        .update_name(OrganizationName::new("UpdatedLifecycle".to_string()).unwrap())
        .unwrap();
    let persisted = repo.update(&updated).await.expect("Update should succeed");
    assert_eq!(persisted.name().as_str(), "UpdatedLifecycle");

    repo.delete(persisted.id())
        .await
        .expect("Delete should succeed");

    let result = repo.find_by_id(persisted.id()).await;
    assert!(result.is_err(), "Organization should be deleted");
}
