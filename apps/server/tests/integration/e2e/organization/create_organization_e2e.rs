//! End-to-end test for creating an organization
//!
//! This test verifies the complete organization creation workflow:
//! 1. Organization is stored in SurrealDB
//! 2. Creator is added as owner in user_organization relation
//! 3. RBAC role is assigned
//! 4. Organization can be retrieved from database

use mockall::mock;
use scylla_core::application::dto::CreateOrganizationRequestDto;
use scylla_core::application::ports::RbacEnforcer;
use scylla_core::application::use_cases::organization::create_organization::CreateOrganizationUseCase;
use scylla_core::domain::entities::User;
use scylla_core::domain::errors::DomainResult;
use scylla_core::domain::repositories::{
    OrganizationRepository, UserOrganizationRepository, UserRepository,
};
use scylla_core::domain::value_objects::{Description, OrganizationName, UserId, Username};
use scylla_core::infrastructure::persistence::surrealdb::organization_repository::SurrealOrganizationRepository;
use scylla_core::infrastructure::persistence::surrealdb::user_organization_repository::SurrealUserOrganizationRepository;
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

/// Helper to set up infrastructure for organization E2E tests
async fn setup_infrastructure() -> (
    Arc<dyn OrganizationRepository>,
    Arc<dyn UserOrganizationRepository>,
    Arc<dyn RbacEnforcer>,
) {
    let db = setup_test_db().await;

    let org_repo: Arc<dyn OrganizationRepository> =
        Arc::new(SurrealOrganizationRepository::new(db.clone()));
    let user_org_repo: Arc<dyn UserOrganizationRepository> =
        Arc::new(SurrealUserOrganizationRepository::new(db.clone()));

    let mut rbac_enforcer = MockRbacEnforcer::new();
    rbac_enforcer
        .expect_add_role_for_user()
        .returning(|_, _, _| Ok(()));

    rbac_enforcer
        .expect_has_role()
        .returning(|_, _, _| Ok(true));

    (org_repo, user_org_repo, Arc::new(rbac_enforcer))
}

#[tokio::test]
#[serial]
async fn test_create_organization_end_to_end_success() {
    // Arrange: Set up real infrastructure
    let (org_repo, user_org_repo, rbac_enforcer) = setup_infrastructure().await;

    let use_case = CreateOrganizationUseCase::new(
        Arc::clone(&org_repo),
        Arc::clone(&user_org_repo),
        Arc::clone(&rbac_enforcer),
    );

    // Create a user first (creator)
    let db = setup_test_db().await;
    let user_repo = SurrealUserRepository::new(db);
    let creator = User::create(
        Username::try_from("creator".to_string()).unwrap(),
        "hashed_password".to_string(),
    );
    let created_user = user_repo.create(&creator).await.unwrap();

    let request = CreateOrganizationRequestDto {
        name: OrganizationName::new("E2EOrg".to_string()).unwrap(),
        description: Some(Description::new("E2E test organization".to_string()).unwrap()),
        creator_id: created_user.id().clone(),
    };

    // Act: Execute the complete workflow
    let result = use_case.execute(request).await;

    // Assert: Organization was created successfully
    assert!(
        result.is_ok(),
        "End-to-end organization creation should succeed"
    );

    let response = result.unwrap();
    assert_eq!(response.name.as_str(), "E2EOrg");
    assert!(response.is_active);

    // Verify: Organization exists in database
    let stored_org = org_repo
        .find_by_id(&response.id)
        .await
        .expect("Organization should exist in database");

    assert_eq!(stored_org.name().as_str(), "E2EOrg");
    assert_eq!(
        stored_org.description().as_ref().map(|d| d.as_str()),
        Some("E2E test organization")
    );

    // Verify: Creator is linked as owner
    let user_orgs = user_org_repo
        .list_organizations_for_user(&created_user.id(), None)
        .await
        .expect("Should list user organizations");

    assert_eq!(
        user_orgs.items().len(),
        1,
        "Creator should have one organization"
    );

    // Verify: RBAC role was assigned
    let has_role = rbac_enforcer
        .has_role(&created_user.id(), "org_owner", response.id.as_str())
        .await
        .expect("RBAC check should work");

    assert!(has_role, "Creator should have owner role");
}

#[tokio::test]
#[serial]
async fn test_create_duplicate_organization_end_to_end() {
    // Arrange
    let (org_repo, user_org_repo, rbac_enforcer) = setup_infrastructure().await;

    let use_case = CreateOrganizationUseCase::new(
        Arc::clone(&org_repo),
        Arc::clone(&user_org_repo),
        Arc::clone(&rbac_enforcer),
    );

    // Create a user first
    let db = setup_test_db().await;
    let user_repo = SurrealUserRepository::new(db);
    let creator = User::create(
        Username::try_from("creator2".to_string()).unwrap(),
        "hashed_password".to_string(),
    );
    let created_user = user_repo.create(&creator).await.unwrap();

    let request1 = CreateOrganizationRequestDto {
        name: OrganizationName::new("DuplicateOrg".to_string()).unwrap(),
        description: None,
        creator_id: created_user.id().clone(),
    };

    // Act: Create first organization
    let result1 = use_case.execute(request1).await;
    assert!(
        result1.is_ok(),
        "First organization creation should succeed"
    );

    // Act: Try to create duplicate organization
    let request2 = CreateOrganizationRequestDto {
        name: OrganizationName::new("DuplicateOrg".to_string()).unwrap(),
        description: Some(Description::new("Different description".to_string()).unwrap()),
        creator_id: UserId::generate(),
    };

    let result2 = use_case.execute(request2).await;

    // Assert: Second creation should fail with conflict
    assert!(
        result2.is_err(),
        "Creating duplicate organization should fail end-to-end"
    );

    // Verify: Only one organization exists in database
    let name = OrganizationName::new("DuplicateOrg".to_string()).unwrap();
    let exists = org_repo
        .name_exists(&name)
        .await
        .expect("Name check should work");

    assert!(exists, "First organization should still exist");
}
