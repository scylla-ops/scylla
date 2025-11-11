//! Integration tests for ListProjectsUseCase

use scylla_core::application::dto::ListProjectsRequestDto;
use scylla_core::application::use_cases::project::list_projects::ListProjectsUseCase;
use scylla_core::domain::entities::{Organization, Project};
use scylla_core::domain::repositories::{OrganizationRepository, ProjectRepository};
use scylla_core::domain::value_objects::{OrganizationName, PaginationParams, ProjectName};
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
async fn test_list_projects_use_case_success() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let project_repo: Arc<dyn ProjectRepository> =
        Arc::new(SurrealProjectRepository::new(db.clone()));
    let use_case = ListProjectsUseCase::new(project_repo.clone());

    let org = create_test_organization("ListOrg");
    let created_org = org_repo.create(&org).await.unwrap();

    // Create multiple projects
    for i in 0..5 {
        let project = create_test_project(&format!("Project{}", i), created_org.id());
        project_repo.create(&project).await.unwrap();
    }

    let request = ListProjectsRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List projects should succeed");
    let response = result.unwrap();
    assert_eq!(response.projects.len(), 5);
    assert!(response.pagination.is_some());
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 5);
    }
}

#[tokio::test]
#[serial]
async fn test_list_projects_use_case_with_pagination() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let project_repo: Arc<dyn ProjectRepository> =
        Arc::new(SurrealProjectRepository::new(db.clone()));
    let use_case = ListProjectsUseCase::new(project_repo.clone());

    let org = create_test_organization("PaginatedOrg");
    let created_org = org_repo.create(&org).await.unwrap();

    // Create 10 projects
    for i in 0..10 {
        let project = create_test_project(&format!("PaginatedProject{}", i), created_org.id());
        project_repo.create(&project).await.unwrap();
    }

    let pagination = PaginationParams::new(1, 3).unwrap();
    let request = ListProjectsRequestDto {
        pagination: Some(pagination),
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List projects with pagination should succeed"
    );
    let response = result.unwrap();
    assert_eq!(response.projects.len(), 3);
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 10);
    }
}

#[tokio::test]
#[serial]
async fn test_list_projects_use_case_empty() {
    let db = setup_test_db().await;
    let project_repo: Arc<dyn ProjectRepository> = Arc::new(SurrealProjectRepository::new(db));
    let use_case = ListProjectsUseCase::new(project_repo);

    let request = ListProjectsRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List projects should succeed even when empty"
    );
    let response = result.unwrap();
    assert_eq!(response.projects.len(), 0);
}
