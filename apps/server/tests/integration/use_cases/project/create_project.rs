//! Integration tests for CreateProjectUseCase
//!
//! These tests verify the CreateProjectUseCase with real repositories.

use mockall::mock;
use scylla_core::application::dto::CreateProjectRequestDto;
use scylla_core::application::ports::RbacEnforcer;
use scylla_core::application::use_cases::project::create_project::CreateProjectUseCase;
use scylla_core::domain::entities::Organization;
use scylla_core::domain::errors::DomainResult;
use scylla_core::domain::repositories::{
    OrganizationRepository, ProjectRepository, UserProjectRepository,
};
use scylla_core::domain::value_objects::{Description, OrganizationName, ProjectName, UserId};
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use scylla_core::infrastructure::persistence::surrealdb::project_repository::SurrealProjectRepository;
use scylla_core::infrastructure::persistence::surrealdb::user_project_repository::SurrealUserProjectRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

// Mock RBAC enforcer for testing
mock! {
    pub RbacEnforcer {}

    #[async_trait::async_trait]
    impl RbacEnforcer for RbacEnforcer {
        async fn enforce(&self, user_id: &UserId, domain: &str, resource: &str, action: &str) -> DomainResult<bool>;
        async fn add_role_for_user(&self, user_id: &UserId, role: &str, domain: &str) -> DomainResult<()>;
        async fn remove_role_for_user(&self, user_id: &UserId, role: &str, domain: &str) -> DomainResult<()>;
        async fn get_roles_for_user(&self, user_id: &UserId, domain: &str) -> DomainResult<Vec<String>>;
        async fn get_users_for_role(&self, role: &str, domain: &str) -> DomainResult<Vec<UserId>>;
        async fn has_role(&self, user_id: &UserId, role: &str, domain: &str) -> DomainResult<bool>;
    }
}

#[tokio::test]
#[serial]
async fn test_create_project_use_case_with_real_repository() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let project_repo: Arc<dyn ProjectRepository> =
        Arc::new(SurrealProjectRepository::new(db.clone()));
    let user_project_repo: Arc<dyn UserProjectRepository> =
        Arc::new(SurrealUserProjectRepository::new(db.clone()));

    // Create an organization first
    let org =
        Organization::create(OrganizationName::new("TestOrg".to_string()).unwrap(), None).unwrap();
    let created_org = org_repo.create(&org).await.unwrap();

    let mut rbac_enforcer = MockRbacEnforcer::new();
    rbac_enforcer
        .expect_add_role_for_user()
        .returning(|_, _, _| Ok(()));

    let use_case = CreateProjectUseCase::new(
        project_repo.clone(),
        user_project_repo,
        Arc::new(rbac_enforcer),
    );

    let creator_id = UserId::generate();
    let request = CreateProjectRequestDto {
        name: ProjectName::new("TestProject".to_string()).unwrap(),
        description: Some(Description::new("Test description".to_string()).unwrap()),
        organization_id: created_org.id().clone(),
        creator_id: creator_id.clone(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Use case should succeed");
    let response = result.unwrap();
    assert_eq!(response.name.as_str(), "TestProject");
    assert!(response.is_active);

    // Verify project exists in repository
    let project = project_repo
        .find_by_id(&response.id)
        .await
        .expect("Project should exist");
    assert_eq!(project.name().as_str(), "TestProject");
}
