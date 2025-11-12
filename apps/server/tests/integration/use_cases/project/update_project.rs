//! Integration tests for UpdateProjectUseCase

use scylla_core::application::dto::UpdateProjectRequestDto;
use scylla_core::application::use_cases::project::update_project::UpdateProjectUseCase;
use scylla_core::domain::entities::{Organization, Project};
use scylla_core::domain::repositories::{OrganizationRepository, ProjectRepository};
use scylla_core::domain::value_objects::{Description, OrganizationName, ProjectName};
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
async fn test_update_project_use_case_update_name() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let project_repo: Arc<dyn ProjectRepository> =
        Arc::new(SurrealProjectRepository::new(db.clone()));
    let use_case = UpdateProjectUseCase::new(project_repo.clone());

    let org = create_test_organization("UpdateOrg");
    let created_org = org_repo.create(&org).await.unwrap();

    let project = create_test_project("Original", created_org.id());
    let created = project_repo.create(&project).await.unwrap();

    let request = UpdateProjectRequestDto {
        project_id: created.id().clone(),
        name: Some(ProjectName::new("UpdatedName".to_string()).unwrap()),
        description: None,
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Update project should succeed");
    let response = result.unwrap();
    assert_eq!(response.name.as_str(), "UpdatedName");

    // Verify it's persisted
    let updated = project_repo.find_by_id(&response.id).await.unwrap();
    assert_eq!(updated.name().as_str(), "UpdatedName");
}

#[tokio::test]
#[serial]
async fn test_update_project_use_case_update_description() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let project_repo: Arc<dyn ProjectRepository> =
        Arc::new(SurrealProjectRepository::new(db.clone()));
    let use_case = UpdateProjectUseCase::new(project_repo.clone());

    let org = create_test_organization("DescOrg");
    let created_org = org_repo.create(&org).await.unwrap();

    let project = create_test_project("DescProject", created_org.id());
    let created = project_repo.create(&project).await.unwrap();

    let request = UpdateProjectRequestDto {
        project_id: created.id().clone(),
        name: None,
        description: Some(Description::new("New description".to_string()).unwrap()),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Update project description should succeed");
    let response = result.unwrap();
    assert_eq!(
        response.description.as_ref().and_then(|d| Some(d.as_str())),
        Some("New description")
    );
}

#[tokio::test]
#[serial]
async fn test_update_project_use_case_not_found() {
    let db = setup_test_db().await;
    let project_repo: Arc<dyn ProjectRepository> = Arc::new(SurrealProjectRepository::new(db));
    let use_case = UpdateProjectUseCase::new(project_repo);

    let request = UpdateProjectRequestDto {
        project_id: scylla_core::domain::value_objects::ProjectId::generate(),
        name: Some(ProjectName::new("NewName".to_string()).unwrap()),
        description: None,
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Update non-existent project should fail");
}
