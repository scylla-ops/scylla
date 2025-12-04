//! Integration tests for CreateOrganizationUseCase
//!
//! These tests verify the CreateOrganizationUseCase with real repositories.

use mockall::mock;
use scylla_core::application::dto::CreateOrganizationRequestDto;
use scylla_core::application::ports::RbacEnforcer;
use scylla_core::application::use_cases::organization::create_organization::CreateOrganizationUseCase;
use scylla_core::domain::errors::DomainResult;
use scylla_core::domain::repositories::{OrganizationRepository, UserOrganizationRepository};
use scylla_core::domain::value_objects::{Description, OrganizationName, UserId};
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use scylla_core::infrastructure::persistence::surrealdb::user_organization_repository::SurrealUserOrganizationRepository;
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
async fn test_create_organization_use_case_with_real_repository() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let user_org_repo: Arc<dyn UserOrganizationRepository> =
        Arc::new(SurrealUserOrganizationRepository::new(db.clone()));

    let mut rbac_enforcer = MockRbacEnforcer::new();
    rbac_enforcer
        .expect_add_role_for_user()
        .returning(|_, _, _| Ok(()));

    let use_case =
        CreateOrganizationUseCase::new(org_repo.clone(), user_org_repo, Arc::new(rbac_enforcer));

    let creator_id = UserId::generate();
    let request = CreateOrganizationRequestDto {
        name: OrganizationName::new("TestOrg".to_string()).unwrap(),
        description: Some(Description::new("Test description".to_string()).unwrap()),
        creator_id: creator_id.clone(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Use case should succeed");
    let response = result.unwrap();
    assert_eq!(response.name.as_str(), "TestOrg");
    assert!(response.is_active);

    // Verify organization exists in repository
    let org = org_repo
        .find_by_id(&response.id)
        .await
        .expect("Organization should exist");
    assert_eq!(org.name().as_str(), "TestOrg");
}

#[tokio::test]
#[serial]
async fn test_create_organization_use_case_duplicate_name() {
    let db = setup_test_db().await;
    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let user_org_repo: Arc<dyn UserOrganizationRepository> =
        Arc::new(SurrealUserOrganizationRepository::new(db.clone()));

    let mut rbac_enforcer = MockRbacEnforcer::new();
    // Expect only one call (for the first organization)
    rbac_enforcer
        .expect_add_role_for_user()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let use_case = CreateOrganizationUseCase::new(
        org_repo.clone(),
        user_org_repo.clone(),
        Arc::new(rbac_enforcer),
    );

    let creator_id = UserId::generate();
    let request1 = CreateOrganizationRequestDto {
        name: OrganizationName::new("Duplicate".to_string()).unwrap(),
        description: None,
        creator_id: creator_id.clone(),
    };

    let result1 = use_case.execute(request1).await;
    assert!(result1.is_ok(), "First creation should succeed");

    // For duplicate test, create a new use case with a mock that doesn't expect RBAC calls
    let rbac_enforcer2 = MockRbacEnforcer::new();
    // RBAC should not be called when duplicate name check fails
    let use_case2 =
        CreateOrganizationUseCase::new(org_repo.clone(), user_org_repo, Arc::new(rbac_enforcer2));

    let request2 = CreateOrganizationRequestDto {
        name: OrganizationName::new("Duplicate".to_string()).unwrap(),
        description: None,
        creator_id: UserId::generate(),
    };

    let result2 = use_case2.execute(request2).await;
    assert!(result2.is_err(), "Duplicate name should fail");
}
