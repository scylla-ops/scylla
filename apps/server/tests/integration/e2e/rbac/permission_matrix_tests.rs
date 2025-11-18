//! E2E tests for RBAC Permission Matrix
//!
//! These tests verify the complete permission matrix as defined in RBAC_PERSONAS_AND_PERMISSIONS.md
//! Tests all role-resource-action combinations across global, organization, and project domains.

use casbin::{CoreApi, Enforcer, MgmtApi};
use scylla_core::application::ports::RbacEnforcer;
use scylla_core::domain::value_objects::UserId;
use scylla_core::infrastructure::rbac::casbin_enforcer::CasbinRbacEnforcer;
use serial_test::serial;
use std::sync::Arc;
use surreal_casbin_adapter::SurrealAdapter;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// Setup test database
async fn setup_test_db() -> Arc<Surreal<Any>> {
    let db = surrealdb::engine::any::connect("mem://")
        .await
        .expect("Failed to create in-memory database");

    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to set namespace/database");

    db.query(
        r#"
        DEFINE TABLE casbin_rules SCHEMAFULL;
        DEFINE FIELD ptype ON casbin_rules TYPE string;
        DEFINE FIELD v0 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v1 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v2 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v3 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v4 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v5 ON casbin_rules TYPE option<string>;
        DEFINE INDEX casbin_ptype_idx ON casbin_rules COLUMNS ptype;
        "#,
    )
    .await
    .expect("Failed to create casbin_rules table");

    Arc::new(db)
}

/// Setup enforcer with ONLY global permission policies
/// Domain-specific policies should be added by individual tests
async fn setup_permission_matrix() -> CasbinRbacEnforcer {
    let db = setup_test_db().await;
    let adapter = SurrealAdapter::new(db, "casbin_rules");

    let mut enforcer: Enforcer = Enforcer::new("casbin/model.conf", adapter)
        .await
        .expect("Failed to create enforcer");

    // Global Admin permissions (domain: "*")
    let admin_policies = [
        ("admin", "*", "organizations", "create"),
        ("admin", "*", "organizations", "read"),
        ("admin", "*", "organizations", "update"),
        ("admin", "*", "organizations", "delete"),
        ("admin", "*", "users", "create"),
        ("admin", "*", "users", "read"),
        ("admin", "*", "users", "update"),
        ("admin", "*", "users", "delete"),
        ("admin", "*", "projects", "create"),
        ("admin", "*", "projects", "read"),
        ("admin", "*", "projects", "update"),
        ("admin", "*", "projects", "delete"),
        ("admin", "*", "pipelines", "create"),
        ("admin", "*", "pipelines", "read"),
        ("admin", "*", "pipelines", "update"),
        ("admin", "*", "pipelines", "delete"),
        ("admin", "*", "pipelines", "execute"),
        ("admin", "*", "jobs", "create"),
        ("admin", "*", "jobs", "read"),
        ("admin", "*", "jobs", "update"),
        ("admin", "*", "jobs", "delete"),
    ];

    // Global User permissions (domain: "*")
    let user_policies = [("user", "*", "organizations", "read")];

    // Organization Owner permissions
    let org_owner_policies = [
        ("org_owner", "org_id", "organizations", "read"),
        ("org_owner", "org_id", "organizations", "update"),
        ("org_owner", "org_id", "organizations", "delete"),
        ("org_owner", "org_id", "organizations", "toggle_active"),
        ("org_owner", "org_id", "users", "add_to_org"),
        ("org_owner", "org_id", "users", "remove_from_org"),
        ("org_owner", "org_id", "users", "list_org_users"),
        ("org_owner", "org_id", "projects", "create"),
        ("org_owner", "org_id", "projects", "read"),
        ("org_owner", "org_id", "projects", "update"),
        ("org_owner", "org_id", "projects", "delete"),
    ];

    // Organization Admin permissions
    let org_admin_policies = [
        ("org_admin", "org_id", "organizations", "read"),
        ("org_admin", "org_id", "organizations", "update"),
        ("org_admin", "org_id", "users", "add_to_org"),
        ("org_admin", "org_id", "users", "remove_from_org"),
        ("org_admin", "org_id", "users", "list_org_users"),
        ("org_admin", "org_id", "projects", "create"),
        ("org_admin", "org_id", "projects", "read"),
        ("org_admin", "org_id", "projects", "update"),
        ("org_admin", "org_id", "projects", "delete"),
    ];

    // Organization Member permissions
    let org_member_policies = [
        ("org_member", "org_id", "organizations", "read"),
        ("org_member", "org_id", "projects", "read"),
    ];

    // Project Owner permissions
    let project_owner_policies = [
        ("project_owner", "project_id", "projects", "read"),
        ("project_owner", "project_id", "projects", "update"),
        ("project_owner", "project_id", "projects", "delete"),
        ("project_owner", "project_id", "projects", "toggle_active"),
        ("project_owner", "project_id", "users", "add_to_project"),
        (
            "project_owner",
            "project_id",
            "users",
            "remove_from_project",
        ),
        ("project_owner", "project_id", "pipelines", "create"),
        ("project_owner", "project_id", "pipelines", "read"),
        ("project_owner", "project_id", "pipelines", "update"),
        ("project_owner", "project_id", "pipelines", "delete"),
        ("project_owner", "project_id", "pipelines", "execute"),
        ("project_owner", "project_id", "jobs", "create"),
        ("project_owner", "project_id", "jobs", "read"),
        ("project_owner", "project_id", "jobs", "update"),
        ("project_owner", "project_id", "jobs", "delete"),
    ];

    // Project Admin permissions
    let project_admin_policies = [
        ("project_admin", "project_id", "projects", "read"),
        ("project_admin", "project_id", "projects", "update"),
        ("project_admin", "project_id", "users", "add_to_project"),
        (
            "project_admin",
            "project_id",
            "users",
            "remove_from_project",
        ),
        ("project_admin", "project_id", "pipelines", "create"),
        ("project_admin", "project_id", "pipelines", "read"),
        ("project_admin", "project_id", "pipelines", "update"),
        ("project_admin", "project_id", "pipelines", "delete"),
        ("project_admin", "project_id", "pipelines", "execute"),
        ("project_admin", "project_id", "jobs", "create"),
        ("project_admin", "project_id", "jobs", "read"),
        ("project_admin", "project_id", "jobs", "update"),
        ("project_admin", "project_id", "jobs", "delete"),
    ];

    // Project Member permissions
    let project_member_policies = [
        ("project_member", "project_id", "projects", "read"),
        ("project_member", "project_id", "pipelines", "read"),
        ("project_member", "project_id", "pipelines", "execute"),
        ("project_member", "project_id", "jobs", "read"),
        ("project_member", "project_id", "jobs", "update"),
    ];

    // Add all policies
    for (role, domain, resource, action) in admin_policies
        .iter()
        .chain(user_policies.iter())
        .chain(org_owner_policies.iter())
        .chain(org_admin_policies.iter())
        .chain(org_member_policies.iter())
        .chain(project_owner_policies.iter())
        .chain(project_admin_policies.iter())
        .chain(project_member_policies.iter())
    {
        let policy: Vec<String> = vec![
            role.to_string(),
            domain.to_string(),
            resource.to_string(),
            action.to_string(),
        ];
        enforcer
            .add_policy(policy)
            .await
            .expect("Failed to add policy");
    }

    CasbinRbacEnforcer::new(enforcer)
}

// ============================================================================
// GLOBAL ADMIN TESTS
// ============================================================================

#[tokio::test]
#[serial]
async fn test_global_admin_can_create_organizations() {
    let enforcer = setup_permission_matrix().await;
    let alice = UserId::new("users:alice".to_string());

    enforcer
        .add_role_for_user(&alice, "admin", "*")
        .await
        .unwrap();

    assert!(
        enforcer
            .enforce(&alice, "*", "organizations", "create")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_global_admin_can_manage_all_users() {
    let enforcer = setup_permission_matrix().await;
    let alice = UserId::new("users:alice".to_string());

    enforcer
        .add_role_for_user(&alice, "admin", "*")
        .await
        .unwrap();

    assert!(
        enforcer
            .enforce(&alice, "*", "users", "create")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&alice, "*", "users", "read")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&alice, "*", "users", "update")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&alice, "*", "users", "delete")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_global_admin_can_manage_all_projects() {
    let enforcer = setup_permission_matrix().await;
    let alice = UserId::new("users:alice".to_string());

    enforcer
        .add_role_for_user(&alice, "admin", "*")
        .await
        .unwrap();

    assert!(
        enforcer
            .enforce(&alice, "*", "projects", "create")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&alice, "*", "projects", "delete")
            .await
            .unwrap()
    );
}

// ============================================================================
// GLOBAL USER TESTS
// ============================================================================

#[tokio::test]
#[serial]
async fn test_global_user_can_only_read_organizations() {
    let enforcer = setup_permission_matrix().await;
    let bob = UserId::new("users:bob".to_string());

    enforcer.add_role_for_user(&bob, "user", "*").await.unwrap();

    // Can read
    assert!(
        enforcer
            .enforce(&bob, "*", "organizations", "read")
            .await
            .unwrap()
    );

    // Cannot create, update, or delete
    assert!(
        !enforcer
            .enforce(&bob, "*", "organizations", "create")
            .await
            .unwrap()
    );
    assert!(
        !enforcer
            .enforce(&bob, "*", "organizations", "update")
            .await
            .unwrap()
    );
    assert!(
        !enforcer
            .enforce(&bob, "*", "organizations", "delete")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_global_user_cannot_manage_users() {
    let enforcer = setup_permission_matrix().await;
    let bob = UserId::new("users:bob".to_string());

    enforcer.add_role_for_user(&bob, "user", "*").await.unwrap();

    assert!(
        !enforcer
            .enforce(&bob, "*", "users", "create")
            .await
            .unwrap()
    );
    assert!(
        !enforcer
            .enforce(&bob, "*", "users", "delete")
            .await
            .unwrap()
    );
}

// ============================================================================
// ORGANIZATION OWNER TESTS
// ============================================================================

#[tokio::test]
#[serial]
async fn test_org_owner_full_organization_control() {
    let enforcer = setup_permission_matrix().await;
    let carol = UserId::new("users:carol".to_string());
    let org_id = "organizations:dataflow";

    enforcer
        .add_role_for_user(&carol, "org_owner", org_id)
        .await
        .unwrap();

    // Full CRUD on organization
    assert!(
        enforcer
            .enforce(&carol, org_id, "organizations", "read")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&carol, org_id, "organizations", "update")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&carol, org_id, "organizations", "delete")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&carol, org_id, "organizations", "toggle_active")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_org_owner_can_manage_org_users() {
    let enforcer = setup_permission_matrix().await;
    let carol = UserId::new("users:carol".to_string());
    let org_id = "organizations:dataflow";

    enforcer
        .add_role_for_user(&carol, "org_owner", org_id)
        .await
        .unwrap();

    assert!(
        enforcer
            .enforce(&carol, org_id, "users", "add_to_org")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&carol, org_id, "users", "remove_from_org")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&carol, org_id, "users", "list_org_users")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_org_owner_can_manage_projects() {
    let enforcer = setup_permission_matrix().await;
    let carol = UserId::new("users:carol".to_string());
    let org_id = "organizations:dataflow";

    enforcer
        .add_role_for_user(&carol, "org_owner", org_id)
        .await
        .unwrap();

    assert!(
        enforcer
            .enforce(&carol, org_id, "projects", "create")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&carol, org_id, "projects", "read")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&carol, org_id, "projects", "update")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&carol, org_id, "projects", "delete")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_org_owner_cannot_access_different_org() {
    let enforcer = setup_permission_matrix().await;
    let carol = UserId::new("users:carol".to_string());

    enforcer
        .add_role_for_user(&carol, "org_owner", "organizations:org1")
        .await
        .unwrap();

    // Should not have access to different organization
    assert!(
        !enforcer
            .enforce(&carol, "organizations:org2", "organizations", "update")
            .await
            .unwrap()
    );
    assert!(
        !enforcer
            .enforce(&carol, "organizations:org2", "projects", "create")
            .await
            .unwrap()
    );
}

// ============================================================================
// ORGANIZATION ADMIN TESTS
// ============================================================================

#[tokio::test]
#[serial]
async fn test_org_admin_can_manage_but_not_delete_org() {
    let enforcer = setup_permission_matrix().await;
    let dave = UserId::new("users:dave".to_string());
    let org_id = "organizations:techcorp";

    enforcer
        .add_role_for_user(&dave, "org_admin", org_id)
        .await
        .unwrap();

    // Can read and update
    assert!(
        enforcer
            .enforce(&dave, org_id, "organizations", "read")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&dave, org_id, "organizations", "update")
            .await
            .unwrap()
    );

    // Cannot delete organization
    assert!(
        !enforcer
            .enforce(&dave, org_id, "organizations", "delete")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_org_admin_can_manage_users_and_projects() {
    let enforcer = setup_permission_matrix().await;
    let dave = UserId::new("users:dave".to_string());
    let org_id = "organizations:techcorp";

    enforcer
        .add_role_for_user(&dave, "org_admin", org_id)
        .await
        .unwrap();

    // Can manage users
    assert!(
        enforcer
            .enforce(&dave, org_id, "users", "add_to_org")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&dave, org_id, "users", "remove_from_org")
            .await
            .unwrap()
    );

    // Can manage projects
    assert!(
        enforcer
            .enforce(&dave, org_id, "projects", "create")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&dave, org_id, "projects", "delete")
            .await
            .unwrap()
    );
}

// ============================================================================
// ORGANIZATION MEMBER TESTS
// ============================================================================

#[tokio::test]
#[serial]
async fn test_org_member_can_only_read() {
    let enforcer = setup_permission_matrix().await;
    let emily = UserId::new("users:emily".to_string());
    let org_id = "organizations:startup";

    enforcer
        .add_role_for_user(&emily, "org_member", org_id)
        .await
        .unwrap();

    // Can read
    assert!(
        enforcer
            .enforce(&emily, org_id, "organizations", "read")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&emily, org_id, "projects", "read")
            .await
            .unwrap()
    );

    // Cannot modify
    assert!(
        !enforcer
            .enforce(&emily, org_id, "organizations", "update")
            .await
            .unwrap()
    );
    assert!(
        !enforcer
            .enforce(&emily, org_id, "projects", "create")
            .await
            .unwrap()
    );
    assert!(
        !enforcer
            .enforce(&emily, org_id, "users", "add_to_org")
            .await
            .unwrap()
    );
}

// ============================================================================
// PROJECT OWNER TESTS
// ============================================================================

#[tokio::test]
#[serial]
async fn test_project_owner_full_project_control() {
    let enforcer = setup_permission_matrix().await;
    let frank = UserId::new("users:frank".to_string());
    let project_id = "projects:platform";

    enforcer
        .add_role_for_user(&frank, "project_owner", project_id)
        .await
        .unwrap();

    // Full project control
    assert!(
        enforcer
            .enforce(&frank, project_id, "projects", "read")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&frank, project_id, "projects", "update")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&frank, project_id, "projects", "delete")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&frank, project_id, "projects", "toggle_active")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_project_owner_can_manage_pipelines_and_jobs() {
    let enforcer = setup_permission_matrix().await;
    let frank = UserId::new("users:frank".to_string());
    let project_id = "projects:platform";

    enforcer
        .add_role_for_user(&frank, "project_owner", project_id)
        .await
        .unwrap();

    // Pipelines
    assert!(
        enforcer
            .enforce(&frank, project_id, "pipelines", "create")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&frank, project_id, "pipelines", "execute")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&frank, project_id, "pipelines", "delete")
            .await
            .unwrap()
    );

    // Jobs
    assert!(
        enforcer
            .enforce(&frank, project_id, "jobs", "create")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&frank, project_id, "jobs", "delete")
            .await
            .unwrap()
    );
}

// ============================================================================
// PROJECT ADMIN TESTS
// ============================================================================

#[tokio::test]
#[serial]
async fn test_project_admin_cannot_delete_project() {
    let enforcer = setup_permission_matrix().await;
    let grace = UserId::new("users:grace".to_string());
    let project_id = "projects:backend";

    enforcer
        .add_role_for_user(&grace, "project_admin", project_id)
        .await
        .unwrap();

    // Can manage
    assert!(
        enforcer
            .enforce(&grace, project_id, "projects", "read")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&grace, project_id, "projects", "update")
            .await
            .unwrap()
    );

    // Cannot delete
    assert!(
        !enforcer
            .enforce(&grace, project_id, "projects", "delete")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_project_admin_can_manage_pipelines() {
    let enforcer = setup_permission_matrix().await;
    let grace = UserId::new("users:grace".to_string());
    let project_id = "projects:backend";

    enforcer
        .add_role_for_user(&grace, "project_admin", project_id)
        .await
        .unwrap();

    assert!(
        enforcer
            .enforce(&grace, project_id, "pipelines", "create")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&grace, project_id, "pipelines", "execute")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&grace, project_id, "pipelines", "delete")
            .await
            .unwrap()
    );
}

// ============================================================================
// PROJECT MEMBER TESTS
// ============================================================================

#[tokio::test]
#[serial]
async fn test_project_member_can_execute_but_not_manage_pipelines() {
    let enforcer = setup_permission_matrix().await;
    let henry = UserId::new("users:henry".to_string());
    let project_id = "projects:api";

    enforcer
        .add_role_for_user(&henry, "project_member", project_id)
        .await
        .unwrap();

    // Can read and execute
    assert!(
        enforcer
            .enforce(&henry, project_id, "pipelines", "read")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&henry, project_id, "pipelines", "execute")
            .await
            .unwrap()
    );

    // Cannot create or delete
    assert!(
        !enforcer
            .enforce(&henry, project_id, "pipelines", "create")
            .await
            .unwrap()
    );
    assert!(
        !enforcer
            .enforce(&henry, project_id, "pipelines", "delete")
            .await
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_project_member_can_read_and_update_jobs() {
    let enforcer = setup_permission_matrix().await;
    let henry = UserId::new("users:henry".to_string());
    let project_id = "projects:api";

    enforcer
        .add_role_for_user(&henry, "project_member", project_id)
        .await
        .unwrap();

    // Can read and update
    assert!(
        enforcer
            .enforce(&henry, project_id, "jobs", "read")
            .await
            .unwrap()
    );
    assert!(
        enforcer
            .enforce(&henry, project_id, "jobs", "update")
            .await
            .unwrap()
    );

    // Cannot delete
    assert!(
        !enforcer
            .enforce(&henry, project_id, "jobs", "delete")
            .await
            .unwrap()
    );
}

// ============================================================================
// MULTI-DOMAIN ACCESS TESTS
// ============================================================================

#[tokio::test]
#[serial]
async fn test_user_with_multiple_roles_across_domains() {
    let enforcer = setup_permission_matrix().await;
    let iris = UserId::new("users:iris".to_string());

    // Iris is a global user, org admin, and project member
    enforcer
        .add_role_for_user(&iris, "user", "*")
        .await
        .unwrap();
    enforcer
        .add_role_for_user(&iris, "org_admin", "organizations:company")
        .await
        .unwrap();
    enforcer
        .add_role_for_user(&iris, "project_member", "projects:web")
        .await
        .unwrap();

    // Global permissions
    assert!(
        enforcer
            .enforce(&iris, "*", "organizations", "read")
            .await
            .unwrap()
    );

    // Org permissions
    assert!(
        enforcer
            .enforce(&iris, "organizations:company", "projects", "create")
            .await
            .unwrap()
    );

    // Project permissions
    assert!(
        enforcer
            .enforce(&iris, "projects:web", "pipelines", "execute")
            .await
            .unwrap()
    );

    // No cross-domain access
    assert!(
        !enforcer
            .enforce(&iris, "organizations:other", "projects", "create")
            .await
            .unwrap()
    );
}
