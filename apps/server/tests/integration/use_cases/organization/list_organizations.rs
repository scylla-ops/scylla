//! Integration tests for ListOrganizationsUseCase

use scylla_core::application::dto::ListOrganizationsRequestDto;
use scylla_core::application::use_cases::organization::list_organizations::ListOrganizationsUseCase;
use scylla_core::domain::entities::Organization;
use scylla_core::domain::repositories::OrganizationRepository;
use scylla_core::domain::value_objects::{OrganizationName, PaginationParams};
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

fn create_test_organization(name: &str) -> Organization {
    Organization::create(OrganizationName::new(name.to_string()).unwrap(), None).unwrap()
}

#[tokio::test]
#[serial]
async fn test_list_organizations_use_case_success() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let use_case = ListOrganizationsUseCase::new(org_repo.clone());

    // Create multiple organizations
    for i in 0..5 {
        let org = create_test_organization(&format!("Org{}", i));
        org_repo.create(&org).await.unwrap();
    }

    let request = ListOrganizationsRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List organizations should succeed");
    let response = result.unwrap();
    assert_eq!(response.organizations.len(), 5);
    assert!(response.pagination.is_some());
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 5);
    }
}

#[tokio::test]
#[serial]
async fn test_list_organizations_use_case_with_pagination() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let use_case = ListOrganizationsUseCase::new(org_repo.clone());

    // Create 10 organizations
    for i in 0..10 {
        let org = create_test_organization(&format!("PaginatedOrg{}", i));
        org_repo.create(&org).await.unwrap();
    }

    let pagination = PaginationParams::new(1, 3).unwrap();
    let request = ListOrganizationsRequestDto {
        pagination: Some(pagination),
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List organizations with pagination should succeed"
    );
    let response = result.unwrap();
    assert_eq!(response.organizations.len(), 3);
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 10);
    }
}

#[tokio::test]
#[serial]
async fn test_list_organizations_use_case_empty() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db));
    let use_case = ListOrganizationsUseCase::new(org_repo);

    let request = ListOrganizationsRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List organizations should succeed even when empty"
    );
    let response = result.unwrap();
    assert_eq!(response.organizations.len(), 0);
}
