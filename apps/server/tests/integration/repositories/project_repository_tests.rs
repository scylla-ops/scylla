//! Integration tests for ProjectRepository
//!
//! These tests verify the ProjectRepository implementation against a real SurrealDB in-memory database.

use scylla_core::domain::entities::{Organization, Project};
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::repositories::{OrganizationRepository, ProjectRepository};
use scylla_core::domain::value_objects::{
    Description, OrganizationId, OrganizationName, ProjectName,
};
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use scylla_core::infrastructure::persistence::surrealdb::project_repository::SurrealProjectRepository;
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

/// Helper to create a test project entity
fn create_test_project(name: &str, description: Option<&str>, org_id: &OrganizationId) -> Project {
    Project::create(
        ProjectName::new(name.to_string()).unwrap(),
        description.map(|d| Description::new(d.to_string()).unwrap()),
        org_id.clone(),
    )
    .unwrap()
}

// ===== CREATE Tests =====

#[tokio::test]
#[serial]
async fn test_create_project_success() {
    let db = setup_test_db().await;
    let org_repo = SurrealOrganizationRepository::new(db.clone());
    let repo = SurrealProjectRepository::new(db);

    let org = create_test_organization("TestOrg", None);
    let created_org = org_repo.create(&org).await.unwrap();

    let project = create_test_project("TestProject", Some("Test description"), created_org.id());

    let created = repo
        .create(&project)
        .await
        .expect("Creating project should succeed");

    assert_eq!(created.name().as_str(), "TestProject");
    assert!(created.is_active());
    assert_eq!(
        created.description().as_ref().map(|d| d.as_str()),
        Some("Test description")
    );
    assert_eq!(
        created.organization_id().as_str(),
        created_org.id().as_str()
    );

    let retrieved = repo
        .find_by_id(created.id())
        .await
        .expect("Should be able to retrieve created project");

    assert_eq!(retrieved.name().as_str(), "TestProject");
}

// ===== FIND_BY_ID Tests =====

#[tokio::test]
#[serial]
async fn test_find_by_id_success() {
    let db = setup_test_db().await;
    let org_repo = SurrealOrganizationRepository::new(db.clone());
    let repo = SurrealProjectRepository::new(db);

    let org = create_test_organization("FindOrg", None);
    let created_org = org_repo.create(&org).await.unwrap();

    let project = create_test_project("FindMe", None, created_org.id());
    let created = repo.create(&project).await.unwrap();

    let found = repo
        .find_by_id(created.id())
        .await
        .expect("Should find project by ID");

    assert_eq!(found.name().as_str(), "FindMe");
}

#[tokio::test]
#[serial]
async fn test_find_by_id_not_found() {
    let db = setup_test_db().await;
    let repo = SurrealProjectRepository::new(db);

    let project_id = scylla_core::domain::value_objects::ProjectId::generate();

    let result = repo.find_by_id(&project_id).await;

    assert!(result.is_err(), "Should return error when not found");
    match result.unwrap_err() {
        DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ===== UPDATE Tests =====

#[tokio::test]
#[serial]
async fn test_update_project_success() {
    let db = setup_test_db().await;
    let org_repo = SurrealOrganizationRepository::new(db.clone());
    let repo = SurrealProjectRepository::new(db);

    let org = create_test_organization("UpdateOrg", None);
    let created_org = org_repo.create(&org).await.unwrap();

    let project = create_test_project("Original", None, created_org.id());
    let created = repo.create(&project).await.unwrap();

    let mut updated = created;
    let new_name = ProjectName::new("Updated".to_string()).unwrap();
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
async fn test_delete_project_success() {
    let db = setup_test_db().await;
    let org_repo = SurrealOrganizationRepository::new(db.clone());
    let repo = SurrealProjectRepository::new(db);

    let org = create_test_organization("DeleteOrg", None);
    let created_org = org_repo.create(&org).await.unwrap();

    let project = create_test_project("DeleteMe", None, created_org.id());
    let created = repo.create(&project).await.unwrap();

    repo.delete(created.id())
        .await
        .expect("Delete should succeed");

    let result = repo.find_by_id(created.id()).await;
    assert!(result.is_err(), "Project should be deleted");
}

#[tokio::test]
#[serial]
async fn test_delete_nonexistent_project() {
    let db = setup_test_db().await;
    let repo = SurrealProjectRepository::new(db);

    let project_id = scylla_core::domain::value_objects::ProjectId::generate();

    let result = repo.delete(&project_id).await;
    assert!(
        result.is_ok(),
        "Deleting non-existent project should not error"
    );
}

// ===== LIST_ALL Tests =====

#[tokio::test]
#[serial]
async fn test_list_all_projects() {
    let db = setup_test_db().await;
    let org_repo = SurrealOrganizationRepository::new(db.clone());
    let repo = SurrealProjectRepository::new(db);

    let org = create_test_organization("ListOrg", None);
    let created_org = org_repo.create(&org).await.unwrap();

    for i in 0..5 {
        let project = create_test_project(&format!("Project{}", i), None, created_org.id());
        repo.create(&project).await.unwrap();
    }

    let result = repo.list_all(None).await.expect("Listing should succeed");

    assert_eq!(result.items().len(), 5);
    assert_eq!(result.metadata().total_count(), 5);
}

// ===== LIST_ACTIVE Tests =====

#[tokio::test]
#[serial]
async fn test_list_active_projects() {
    let db = setup_test_db().await;
    let org_repo = SurrealOrganizationRepository::new(db.clone());
    let repo = SurrealProjectRepository::new(db);

    let org = create_test_organization("ActiveOrg", None);
    let created_org = org_repo.create(&org).await.unwrap();

    for i in 0..3 {
        let project = create_test_project(&format!("ActiveProject{}", i), None, created_org.id());
        let created = repo.create(&project).await.unwrap();
        if i == 1 {
            let mut project_draft = repo.find_by_id(&created.id()).await.unwrap();
            project_draft.toggle_active().unwrap();
            repo.update(&project_draft).await.unwrap();
        }
    }

    let result = repo
        .list_active(None)
        .await
        .expect("Listing active should succeed");

    assert_eq!(result.items().len(), 2, "Should have 2 active projects");
}

// ===== Complex Scenarios =====

#[tokio::test]
#[serial]
async fn test_project_full_lifecycle() {
    let db = setup_test_db().await;
    let org_repo = SurrealOrganizationRepository::new(db.clone());
    let repo = SurrealProjectRepository::new(db);

    let org = create_test_organization("LifecycleOrg", None);
    let created_org = org_repo.create(&org).await.unwrap();

    let project = create_test_project("Lifecycle", Some("Initial"), created_org.id());
    let created = repo.create(&project).await.expect("Create should succeed");
    assert!(created.is_active());

    let found = repo
        .find_by_id(created.id())
        .await
        .expect("Find should succeed");
    assert_eq!(found.name().as_str(), "Lifecycle");

    let mut updated = found;
    updated
        .update_name(ProjectName::new("UpdatedLifecycle".to_string()).unwrap())
        .unwrap();
    let persisted = repo.update(&updated).await.expect("Update should succeed");
    assert_eq!(persisted.name().as_str(), "UpdatedLifecycle");

    repo.delete(persisted.id())
        .await
        .expect("Delete should succeed");

    let result = repo.find_by_id(persisted.id()).await;
    assert!(result.is_err(), "Project should be deleted");
}
