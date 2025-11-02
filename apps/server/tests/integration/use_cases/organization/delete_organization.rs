//! Integration tests for DeleteOrganizationUseCase

use scylla_core::application::dto::DeleteOrganizationRequestDto;
use scylla_core::application::use_cases::organization::delete_organization::DeleteOrganizationUseCase;
use scylla_core::domain::entities::Organization;
use scylla_core::domain::repositories::OrganizationRepository;
use scylla_core::domain::value_objects::OrganizationName;
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

fn create_test_organization(name: &str) -> Organization {
    Organization::create(OrganizationName::new(name.to_string()).unwrap(), None).unwrap()
}

#[tokio::test]
#[serial]
async fn test_delete_organization_use_case_success() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let use_case = DeleteOrganizationUseCase::new(org_repo.clone());

    let org = create_test_organization("ToDelete");
    let created = org_repo.create(&org).await.unwrap();

    let request = DeleteOrganizationRequestDto {
        organization_id: created.id().clone(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Delete organization should succeed");

    // Verify organization is deleted
    let get_result = org_repo.find_by_id(created.id()).await;
    assert!(get_result.is_err(), "Organization should be deleted");
}

#[tokio::test]
#[serial]
async fn test_delete_organization_use_case_not_found() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db));
    let use_case = DeleteOrganizationUseCase::new(org_repo);

    let request = DeleteOrganizationRequestDto {
        organization_id: scylla_core::domain::value_objects::OrganizationId::generate(),
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_err(),
        "Delete non-existent organization should fail"
    );
    match result.unwrap_err() {
        scylla_core::domain::errors::DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}
