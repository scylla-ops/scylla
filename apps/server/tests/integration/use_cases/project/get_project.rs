//! Integration tests for GetProjectUseCase

use scylla_core::application::dto::GetProjectRequestDto;
use scylla_core::application::use_cases::project::get_project::GetProjectUseCase;
use scylla_core::domain::entities::{Organization, Project};
use scylla_core::domain::repositories::{OrganizationRepository, ProjectRepository};
use scylla_core::domain::value_objects::{OrganizationName, ProjectName};
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use scylla_core::infrastructure::persistence::surrealdb::project_repository::SurrealProjectRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

fn create_test_organization(name: &str) -> Organization {
    Organization::create(OrganizationName::new(name.to_string()).unwrap(), None).unwrap()
}

fn create_test_project(
    name: &str,
    org_id: &scylla_core::domain::value_objects::OrganizationId,
) -> Project {
    Project::create(
        ProjectName::new(name.to_string()).unwrap(),
        None,
        org_id.clone(),
    )
    .unwrap()
}

#[tokio::test]
#[serial]
async fn test_get_project_use_case_success() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let project_repo: Arc<dyn ProjectRepository> =
        Arc::new(SurrealProjectRepository::new(db.clone()));
    let use_case = GetProjectUseCase::new(project_repo.clone());

    let org = create_test_organization("GetOrg");
    let created_org = org_repo.create(&org).await.unwrap();

    let project = create_test_project("GetProject", created_org.id());
    let created = project_repo.create(&project).await.unwrap();

    let request = GetProjectRequestDto {
        project_id: created.id().clone(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Get project should succeed");
    let response = result.unwrap();
    assert_eq!(response.name.as_str(), "GetProject");
    assert!(response.is_active);
}

#[tokio::test]
#[serial]
async fn test_get_project_use_case_not_found() {
    let db = setup_test_db().await;
    let project_repo: Arc<dyn ProjectRepository> = Arc::new(SurrealProjectRepository::new(db));
    let use_case = GetProjectUseCase::new(project_repo);

    let request = GetProjectRequestDto {
        project_id: scylla_core::domain::value_objects::ProjectId::generate(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Get non-existent project should fail");
    match result.unwrap_err() {
        scylla_core::domain::errors::DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}
