//! End-to-end test for creating a project
//!
//! This test verifies the complete project creation workflow:
//! 1. Project is stored in SurrealDB
//! 2. Creator is added as owner in user_project relation
//! 3. RBAC role is assigned
//! 4. Project can be retrieved from database

use mockall::mock;
use scylla_core::application::dto::CreateProjectRequestDto;
use scylla_core::application::ports::RbacEnforcer;
use scylla_core::application::use_cases::project::create_project::CreateProjectUseCase;
use scylla_core::domain::entities::{Organization, User};
use scylla_core::domain::errors::DomainResult;
use scylla_core::domain::repositories::{
    OrganizationRepository, ProjectRepository, UserProjectRepository, UserRepository,
};
use scylla_core::domain::value_objects::{
    Description, OrganizationName, ProjectName, UserId, Username,
};
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use scylla_core::infrastructure::persistence::surrealdb::project_repository::SurrealProjectRepository;
use scylla_core::infrastructure::persistence::surrealdb::user_project_repository::SurrealUserProjectRepository;
use scylla_core::infrastructure::persistence::surrealdb::user_repository::SurrealUserRepository;
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

/// Helper to set up infrastructure for project E2E tests
async fn setup_infrastructure() -> (
    Arc<dyn OrganizationRepository>,
    Arc<dyn ProjectRepository>,
    Arc<dyn UserProjectRepository>,
    Arc<dyn RbacEnforcer>,
) {
    let db = setup_test_db().await;

    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let project_repo: Arc<dyn ProjectRepository> =
        Arc::new(SurrealProjectRepository::new(db.clone()));
    let user_project_repo: Arc<dyn UserProjectRepository> =
        Arc::new(SurrealUserProjectRepository::new(db.clone()));

    let mut rbac_enforcer = MockRbacEnforcer::new();
    rbac_enforcer
        .expect_add_role_for_user()
        .returning(|_, _, _| Ok(()));

    rbac_enforcer
        .expect_has_role()
        .returning(|_, _, _| Ok(true));

    (
        org_repo,
        project_repo,
        user_project_repo,
        Arc::new(rbac_enforcer),
    )
}

#[tokio::test]
#[serial]
async fn test_create_project_end_to_end_success() {
    // Arrange: Set up real infrastructure
    let (org_repo, project_repo, user_project_repo, rbac_enforcer) = setup_infrastructure().await;

    let use_case = CreateProjectUseCase::new(
        Arc::clone(&project_repo),
        Arc::clone(&user_project_repo),
        Arc::clone(&rbac_enforcer),
    );

    // Create a user and organization first
    let db = setup_test_db().await;
    let user_repo = SurrealUserRepository::new(db.clone());
    let creator = User::create(
        Username::try_from("creator".to_string()).unwrap(),
        "hashed_password".to_string(),
    );
    let created_user = user_repo.create(&creator).await.unwrap();

    let org =
        Organization::create(OrganizationName::new("TestOrg".to_string()).unwrap(), None).unwrap();
    let created_org = org_repo.create(&org).await.unwrap();

    let request = CreateProjectRequestDto {
        name: ProjectName::new("E2EProject".to_string()).unwrap(),
        description: Some(Description::new("E2E test project".to_string()).unwrap()),
        organization_id: created_org.id().clone(),
        creator_id: created_user.id().clone(),
    };

    // Act: Execute the complete workflow
    let result = use_case.execute(request).await;

    // Assert: Project was created successfully
    assert!(result.is_ok(), "End-to-end project creation should succeed");

    let response = result.unwrap();
    assert_eq!(response.name.as_str(), "E2EProject");
    assert!(response.is_active);

    // Verify: Project exists in database
    let stored_project = project_repo
        .find_by_id(&response.id)
        .await
        .expect("Project should exist in database");

    assert_eq!(stored_project.name().as_str(), "E2EProject");
    assert_eq!(
        stored_project.description().as_ref().map(|d| d.as_str()),
        Some("E2E test project")
    );
    assert_eq!(
        stored_project.organization_id().as_str(),
        created_org.id().as_str()
    );

    // Verify: Creator is linked as owner
    let user_projects = user_project_repo
        .list_projects_for_user(&created_user.id(), None)
        .await
        .expect("Should list user projects");

    assert_eq!(
        user_projects.items().len(),
        1,
        "Creator should have one project"
    );

    // Verify: RBAC role was assigned
    let has_role = rbac_enforcer
        .has_role(&created_user.id(), "project_owner", response.id.as_str())
        .await
        .expect("RBAC check should work");

    assert!(has_role, "Creator should have owner role");
}
