//! Integration tests for CasbinRbacEnforcer
//!
//! These tests verify the RBAC enforcer works correctly with real Casbin and SurrealDB.

use casbin::{CoreApi, Enforcer, MgmtApi, RbacApi};
use scylla_core::application::ports::RbacEnforcer;
use scylla_core::domain::value_objects::UserId;
use scylla_core::infrastructure::rbac::casbin_enforcer::CasbinRbacEnforcer;
use serial_test::serial;
use std::sync::Arc;
use surreal_casbin_adapter::SurrealAdapter;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// Creates an in-memory SurrealDB instance for testing
async fn setup_test_db() -> Arc<Surreal<Any>> {
    let db = surrealdb::engine::any::connect("mem://")
        .await
        .expect("Failed to create in-memory database");

    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to set namespace/database");

    // Initialize casbin_rules table
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

/// Creates a CasbinRbacEnforcer with test database and global bootstrap policies only
/// Domain-specific policies should be added by individual tests
async fn setup_enforcer() -> CasbinRbacEnforcer {
    let db = setup_test_db().await;
    let adapter = SurrealAdapter::new(db, "casbin_rules");

    let mut enforcer: Enforcer = Enforcer::new("casbin/model.conf", adapter)
        .await
        .expect("Failed to create enforcer");

    // Bootstrap with ONLY global policies (domain: "*")
    // In a real system, org/project-specific policies are added dynamically
    let bootstrap_policies = [
        ("admin", "*", "organizations", "create"),
        ("admin", "*", "organizations", "read"),
        ("admin", "*", "organizations", "update"),
        ("admin", "*", "organizations", "delete"),
        ("admin", "*", "users", "create"),
        ("admin", "*", "users", "read"),
        ("admin", "*", "users", "update"),
        ("admin", "*", "users", "delete"),
        ("user", "*", "organizations", "read"),
    ];

    for (role, domain, resource, action) in bootstrap_policies {
        let policy: Vec<String> = vec![
            role.to_string(),
            domain.to_string(),
            resource.to_string(),
            action.to_string(),
        ];
        enforcer
            .add_policy(policy)
            .await
            .expect("Failed to add bootstrap policy");
    }

    CasbinRbacEnforcer::new(enforcer)
}

/// Helper to setup enforcer with specific domain policies for testing
/// This adds both global bootstrap policies and test-specific domain policies
async fn setup_enforcer_with_policies(
    policies: Vec<(&str, &str, &str, &str)>,
) -> CasbinRbacEnforcer {
    let db = setup_test_db().await;
    let adapter = SurrealAdapter::new(db, "casbin_rules");

    let mut enforcer: Enforcer = Enforcer::new("casbin/model.conf", adapter)
        .await
        .expect("Failed to create enforcer");

    // Add global bootstrap policies
    let bootstrap_policies = [
        ("admin", "*", "organizations", "create"),
        ("admin", "*", "organizations", "read"),
        ("admin", "*", "organizations", "update"),
        ("admin", "*", "organizations", "delete"),
        ("admin", "*", "users", "create"),
        ("admin", "*", "users", "read"),
        ("admin", "*", "users", "update"),
        ("admin", "*", "users", "delete"),
        ("user", "*", "organizations", "read"),
    ];

    for (role, domain, resource, action) in bootstrap_policies {
        let policy: Vec<String> = vec![
            role.to_string(),
            domain.to_string(),
            resource.to_string(),
            action.to_string(),
        ];
        enforcer
            .add_policy(policy)
            .await
            .expect("Failed to add bootstrap policy");
    }

    // Add test-specific domain policies
    for (role, domain, resource, action) in policies {
        enforcer
            .add_policy(vec![
                role.to_string(),
                domain.to_string(),
                resource.to_string(),
                action.to_string(),
            ])
            .await
            .expect("Failed to add test policy");
    }

    CasbinRbacEnforcer::new(enforcer)
}

#[tokio::test]
#[serial]
async fn test_enforce_global_admin_can_create_organizations() {
    let enforcer = setup_enforcer().await;

    let user_id = UserId::new("users:alice".to_string());

    // Add alice as global admin
    enforcer
        .add_role_for_user(&user_id, "admin", "*")
        .await
        .expect("Failed to add role");

    // Test enforcement
    let allowed = enforcer
        .enforce(&user_id, "*", "organizations", "create")
        .await
        .expect("Enforcement failed");

    assert!(
        allowed,
        "Global admin should be able to create organizations"
    );
}

#[tokio::test]
#[serial]
async fn test_enforce_global_user_cannot_create_organizations() {
    let enforcer = setup_enforcer().await;

    let user_id = UserId::new("users:bob".to_string());

    // Add bob as regular user
    enforcer
        .add_role_for_user(&user_id, "user", "*")
        .await
        .expect("Failed to add role");

    // Test enforcement
    let allowed = enforcer
        .enforce(&user_id, "*", "organizations", "create")
        .await
        .expect("Enforcement failed");

    assert!(
        !allowed,
        "Regular user should NOT be able to create organizations"
    );
}

#[tokio::test]
#[serial]
async fn test_enforce_org_owner_can_create_projects() {
    let org_id = "organizations:dataflow";

    // Setup enforcer with org-specific policies
    let enforcer = setup_enforcer_with_policies(vec![
        ("org_owner", org_id, "projects", "create"),
        ("org_owner", org_id, "organizations", "read"),
        ("org_owner", org_id, "organizations", "update"),
    ])
    .await;

    let user_id = UserId::new("users:carol".to_string());

    // Add carol as org owner for specific organization
    enforcer
        .add_role_for_user(&user_id, "org_owner", org_id)
        .await
        .expect("Failed to add role");

    // Test enforcement
    let allowed = enforcer
        .enforce(&user_id, org_id, "projects", "create")
        .await
        .expect("Enforcement failed");

    assert!(
        allowed,
        "Org owner should be able to create projects in their org"
    );
}

#[tokio::test]
#[serial]
async fn test_enforce_org_owner_cannot_access_different_org() {
    let enforcer = setup_enforcer().await;

    let user_id = UserId::new("users:dave".to_string());
    let org1_id = "organizations:org1";
    let org2_id = "organizations:org2";

    // Add dave as org owner for org1
    enforcer
        .add_role_for_user(&user_id, "org_owner", org1_id)
        .await
        .expect("Failed to add role");

    // Test enforcement - should not have access to org2
    let allowed = enforcer
        .enforce(&user_id, org2_id, "projects", "create")
        .await
        .expect("Enforcement failed");

    assert!(
        !allowed,
        "Org owner should NOT have access to different organization"
    );
}

#[tokio::test]
#[serial]
async fn test_enforce_project_member_can_execute_pipelines() {
    let project_id = "projects:platform";

    // Setup enforcer with project-specific policies
    let enforcer = setup_enforcer_with_policies(vec![
        ("project_member", project_id, "pipelines", "read"),
        ("project_member", project_id, "pipelines", "execute"),
        ("project_owner", project_id, "pipelines", "delete"),
    ])
    .await;

    let user_id = UserId::new("users:emily".to_string());

    // Add emily as project member
    enforcer
        .add_role_for_user(&user_id, "project_member", project_id)
        .await
        .expect("Failed to add role");

    // Test enforcement - member can execute
    let can_execute = enforcer
        .enforce(&user_id, project_id, "pipelines", "execute")
        .await
        .expect("Enforcement failed");

    assert!(
        can_execute,
        "Project member should be able to execute pipelines"
    );

    // Test enforcement - member cannot delete
    let can_delete = enforcer
        .enforce(&user_id, project_id, "pipelines", "delete")
        .await
        .expect("Enforcement failed");

    assert!(
        !can_delete,
        "Project member should NOT be able to delete pipelines"
    );
}

#[tokio::test]
#[serial]
async fn test_add_role_for_user() {
    let enforcer = setup_enforcer().await;

    let user_id = UserId::new("users:frank".to_string());

    // Add role
    let result = enforcer.add_role_for_user(&user_id, "admin", "*").await;

    assert!(result.is_ok(), "Should successfully add role");

    // Verify role was added by checking enforcement
    let allowed = enforcer
        .enforce(&user_id, "*", "users", "create")
        .await
        .expect("Enforcement failed");

    assert!(allowed, "User with admin role should have permissions");
}

#[tokio::test]
#[serial]
async fn test_remove_role_for_user() {
    let enforcer = setup_enforcer().await;

    let user_id = UserId::new("users:grace".to_string());

    // Add role
    enforcer
        .add_role_for_user(&user_id, "admin", "*")
        .await
        .expect("Failed to add role");

    // Verify role is active
    let allowed_before = enforcer
        .enforce(&user_id, "*", "users", "create")
        .await
        .expect("Enforcement failed");
    assert!(allowed_before, "User should have admin permissions");

    // Remove role
    let result = enforcer.remove_role_for_user(&user_id, "admin", "*").await;

    assert!(result.is_ok(), "Should successfully remove role");

    // Verify role was removed
    let allowed_after = enforcer
        .enforce(&user_id, "*", "users", "create")
        .await
        .expect("Enforcement failed");

    assert!(
        !allowed_after,
        "User should no longer have admin permissions after role removal"
    );
}

#[tokio::test]
#[serial]
async fn test_get_roles_for_user() {
    let enforcer = setup_enforcer().await;

    let user_id = UserId::new("users:henry".to_string());
    let org_id = "organizations:acme";

    // Add multiple roles for the user in the same domain
    enforcer
        .add_role_for_user(&user_id, "org_owner", org_id)
        .await
        .expect("Failed to add org_owner role");

    // Get roles
    let roles = enforcer
        .get_roles_for_user(&user_id, org_id)
        .await
        .expect("Failed to get roles");

    assert_eq!(roles.len(), 1, "Should have 1 role");
    assert!(
        roles.contains(&"org_owner".to_string()),
        "Should have org_owner role"
    );
}

#[tokio::test]
#[serial]
async fn test_get_roles_for_user_across_domains() {
    let enforcer = setup_enforcer().await;

    let user_id = UserId::new("users:iris".to_string());
    let org_id = "organizations:techcorp";
    let project_id = "projects:api";

    // Add roles in different domains
    enforcer
        .add_role_for_user(&user_id, "org_admin", org_id)
        .await
        .expect("Failed to add org role");
    enforcer
        .add_role_for_user(&user_id, "project_owner", project_id)
        .await
        .expect("Failed to add project role");

    // Get roles for org domain
    let org_roles = enforcer
        .get_roles_for_user(&user_id, org_id)
        .await
        .expect("Failed to get org roles");

    assert_eq!(org_roles.len(), 1);
    assert!(org_roles.contains(&"org_admin".to_string()));

    // Get roles for project domain
    let project_roles = enforcer
        .get_roles_for_user(&user_id, project_id)
        .await
        .expect("Failed to get project roles");

    assert_eq!(project_roles.len(), 1);
    assert!(project_roles.contains(&"project_owner".to_string()));
}

#[tokio::test]
#[serial]
async fn test_get_users_for_role() {
    let enforcer = setup_enforcer().await;

    let user1 = UserId::new("users:jack".to_string());
    let user2 = UserId::new("users:jill".to_string());
    let org_id = "organizations:startup";

    // Add multiple users to the same role
    enforcer
        .add_role_for_user(&user1, "org_member", org_id)
        .await
        .expect("Failed to add user1");
    enforcer
        .add_role_for_user(&user2, "org_member", org_id)
        .await
        .expect("Failed to add user2");

    // Get users for role
    let users = enforcer
        .get_users_for_role("org_member", org_id)
        .await
        .expect("Failed to get users");

    assert_eq!(users.len(), 2, "Should have 2 users with org_member role");
    assert!(users.contains(&user1));
    assert!(users.contains(&user2));
}

#[tokio::test]
#[serial]
async fn test_has_role() {
    let enforcer = setup_enforcer().await;

    let user_id = UserId::new("users:kate".to_string());
    let project_id = "projects:backend";

    // Initially should not have role
    let has_role_before = enforcer
        .has_role(&user_id, "project_admin", project_id)
        .await
        .expect("Failed to check role");

    assert!(!has_role_before, "User should not have role initially");

    // Add role
    enforcer
        .add_role_for_user(&user_id, "project_admin", project_id)
        .await
        .expect("Failed to add role");

    // Should now have role
    let has_role_after = enforcer
        .has_role(&user_id, "project_admin", project_id)
        .await
        .expect("Failed to check role");

    assert!(has_role_after, "User should have role after assignment");
}

#[tokio::test]
#[serial]
async fn test_has_role_domain_isolation() {
    let enforcer = setup_enforcer().await;

    let user_id = UserId::new("users:leo".to_string());
    let project1 = "projects:frontend";
    let project2 = "projects:backend";

    // Add role in project1
    enforcer
        .add_role_for_user(&user_id, "project_owner", project1)
        .await
        .expect("Failed to add role");

    // Should have role in project1
    let has_role_in_p1 = enforcer
        .has_role(&user_id, "project_owner", project1)
        .await
        .expect("Failed to check role");

    assert!(has_role_in_p1, "User should have role in project1");

    // Should NOT have role in project2
    let has_role_in_p2 = enforcer
        .has_role(&user_id, "project_owner", project2)
        .await
        .expect("Failed to check role");

    assert!(
        !has_role_in_p2,
        "User should NOT have role in project2 (different domain)"
    );
}

#[tokio::test]
#[serial]
async fn test_enforce_multiple_roles_same_domain() {
    let org_id = "organizations:bigcorp";

    // Setup enforcer with org-specific policies for both member and owner roles
    let enforcer = setup_enforcer_with_policies(vec![
        ("org_member", org_id, "organizations", "read"),
        ("org_owner", org_id, "organizations", "read"),
        ("org_owner", org_id, "projects", "create"),
    ])
    .await;

    let user_id = UserId::new("users:maria".to_string());

    // User can have multiple roles in theory, but typically just one per domain
    // This tests that even with org_member, if they also have org_owner, they get owner permissions
    enforcer
        .add_role_for_user(&user_id, "org_member", org_id)
        .await
        .expect("Failed to add member role");
    enforcer
        .add_role_for_user(&user_id, "org_owner", org_id)
        .await
        .expect("Failed to add owner role");

    // Should have owner permissions (can create projects)
    let can_create_projects = enforcer
        .enforce(&user_id, org_id, "projects", "create")
        .await
        .expect("Enforcement failed");

    assert!(
        can_create_projects,
        "User with org_owner role should be able to create projects"
    );

    // Should also have member permissions (can read org)
    let can_read_org = enforcer
        .enforce(&user_id, org_id, "organizations", "read")
        .await
        .expect("Enforcement failed");

    assert!(can_read_org, "User should also have member permissions");
}
