//! Integration tests for UpdateOrganizationUseCase

use scylla_core::application::dto::UpdateOrganizationRequestDto;
use scylla_core::application::use_cases::organization::update_organization::UpdateOrganizationUseCase;
use scylla_core::domain::entities::Organization;
use scylla_core::domain::repositories::OrganizationRepository;
use scylla_core::domain::value_objects::{Description, OrganizationName};
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

fn create_test_organization(name: &str) -> Organization {
    Organization::create(OrganizationName::new(name.to_string()).unwrap(), None).unwrap()
}

#[tokio::test]
#[serial]
async fn test_update_organization_use_case_update_name() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let use_case = UpdateOrganizationUseCase::new(org_repo.clone());

    let org = create_test_organization("Original");
    let created = org_repo.create(&org).await.unwrap();

    let request = UpdateOrganizationRequestDto {
        organization_id: created.id().clone(),
        name: Some(OrganizationName::new("UpdatedName".to_string()).unwrap()),
        description: None,
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Update organization should succeed");
    let response = result.unwrap();
    assert_eq!(response.name.as_str(), "UpdatedName");

    // Verify it's persisted
    let updated = org_repo.find_by_id(&response.id).await.unwrap();
    assert_eq!(updated.name().as_str(), "UpdatedName");
}

#[tokio::test]
#[serial]
async fn test_update_organization_use_case_update_description() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let use_case = UpdateOrganizationUseCase::new(org_repo.clone());

    let org = create_test_organization("DescOrg");
    let created = org_repo.create(&org).await.unwrap();

    let request = UpdateOrganizationRequestDto {
        organization_id: created.id().clone(),
        name: None,
        description: Some(Description::new("New description".to_string()).unwrap()),
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "Update organization description should succeed"
    );
    let response = result.unwrap();
    assert_eq!(
        response.description.as_ref().and_then(|d| Some(d.as_str())),
        Some("New description")
    );
}

#[tokio::test]
#[serial]
async fn test_update_organization_use_case_duplicate_name() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db));
    let use_case = UpdateOrganizationUseCase::new(org_repo.clone());

    let org1 = create_test_organization("Org1");
    let org2 = create_test_organization("Org2");
    let _created1 = org_repo.create(&org1).await.unwrap();
    let created2 = org_repo.create(&org2).await.unwrap();

    // Try to update org2's name to org1's name
    let request = UpdateOrganizationRequestDto {
        organization_id: created2.id().clone(),
        name: Some(OrganizationName::new("Org1".to_string()).unwrap()),
        description: None,
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Update with duplicate name should fail");
    match result.unwrap_err() {
        scylla_core::domain::errors::DomainError::Conflict(_) => {}
        other => panic!("Expected Conflict error, got {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn test_update_organization_use_case_not_found() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db));
    let use_case = UpdateOrganizationUseCase::new(org_repo);

    let request = UpdateOrganizationRequestDto {
        organization_id: scylla_core::domain::value_objects::OrganizationId::generate(),
        name: Some(OrganizationName::new("NewName".to_string()).unwrap()),
        description: None,
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_err(),
        "Update non-existent organization should fail"
    );
}
