//! Unit tests for RoleMapper
//!
//! Tests the mapping between domain role value objects and Casbin role strings.

use scylla_core::domain::value_objects::{UserGlobalRole, UserOrganizationRole, UserProjectRole};
use scylla_core::infrastructure::rbac::role_mapper::RoleMapper;

#[test]
fn test_global_role_to_casbin_admin() {
    let role = UserGlobalRole::admin();
    let casbin_role = RoleMapper::global_role_to_casbin(&role);
    assert_eq!(casbin_role, "admin");
}

#[test]
fn test_global_role_to_casbin_user() {
    let role = UserGlobalRole::user();
    let casbin_role = RoleMapper::global_role_to_casbin(&role);
    assert_eq!(casbin_role, "user");
}

#[test]
fn test_global_role_to_casbin_from_string() {
    let role = UserGlobalRole::new("admin").unwrap();
    let casbin_role = RoleMapper::global_role_to_casbin(&role);
    assert_eq!(casbin_role, "admin");

    let role = UserGlobalRole::new("user").unwrap();
    let casbin_role = RoleMapper::global_role_to_casbin(&role);
    assert_eq!(casbin_role, "user");
}

#[test]
fn test_org_role_to_casbin_owner() {
    let role = UserOrganizationRole::owner();
    let casbin_role = RoleMapper::org_role_to_casbin(&role);
    assert_eq!(casbin_role, "org_owner");
}

#[test]
fn test_org_role_to_casbin_admin() {
    let role = UserOrganizationRole::admin();
    let casbin_role = RoleMapper::org_role_to_casbin(&role);
    assert_eq!(casbin_role, "org_admin");
}

#[test]
fn test_org_role_to_casbin_member() {
    let role = UserOrganizationRole::new("member").unwrap();
    let casbin_role = RoleMapper::org_role_to_casbin(&role);
    assert_eq!(casbin_role, "org_member");
}

#[test]
fn test_org_role_to_casbin_from_string() {
    let role = UserOrganizationRole::new("owner").unwrap();
    let casbin_role = RoleMapper::org_role_to_casbin(&role);
    assert_eq!(casbin_role, "org_owner");

    let role = UserOrganizationRole::new("admin").unwrap();
    let casbin_role = RoleMapper::org_role_to_casbin(&role);
    assert_eq!(casbin_role, "org_admin");

    let role = UserOrganizationRole::new("member").unwrap();
    let casbin_role = RoleMapper::org_role_to_casbin(&role);
    assert_eq!(casbin_role, "org_member");
}

#[test]
fn test_project_role_to_casbin_owner() {
    let role = UserProjectRole::owner();
    let casbin_role = RoleMapper::project_role_to_casbin(&role);
    assert_eq!(casbin_role, "project_owner");
}

#[test]
fn test_project_role_to_casbin_admin() {
    let role = UserProjectRole::admin();
    let casbin_role = RoleMapper::project_role_to_casbin(&role);
    assert_eq!(casbin_role, "project_admin");
}

#[test]
fn test_project_role_to_casbin_member() {
    let role = UserProjectRole::new("member").unwrap();
    let casbin_role = RoleMapper::project_role_to_casbin(&role);
    assert_eq!(casbin_role, "project_member");
}

#[test]
fn test_project_role_to_casbin_from_string() {
    let role = UserProjectRole::new("owner").unwrap();
    let casbin_role = RoleMapper::project_role_to_casbin(&role);
    assert_eq!(casbin_role, "project_owner");

    let role = UserProjectRole::new("admin").unwrap();
    let casbin_role = RoleMapper::project_role_to_casbin(&role);
    assert_eq!(casbin_role, "project_admin");

    let role = UserProjectRole::new("member").unwrap();
    let casbin_role = RoleMapper::project_role_to_casbin(&role);
    assert_eq!(casbin_role, "project_member");
}

#[test]
fn test_role_mapper_consistency() {
    // Test that all role types consistently map to expected Casbin role names

    // Global roles
    assert_eq!(
        RoleMapper::global_role_to_casbin(&UserGlobalRole::admin()),
        "admin"
    );
    assert_eq!(
        RoleMapper::global_role_to_casbin(&UserGlobalRole::user()),
        "user"
    );

    // Org roles with prefixes
    assert_eq!(
        RoleMapper::org_role_to_casbin(&UserOrganizationRole::owner()),
        "org_owner"
    );
    assert_eq!(
        RoleMapper::org_role_to_casbin(&UserOrganizationRole::admin()),
        "org_admin"
    );

    // Project roles with prefixes
    assert_eq!(
        RoleMapper::project_role_to_casbin(&UserProjectRole::owner()),
        "project_owner"
    );
    assert_eq!(
        RoleMapper::project_role_to_casbin(&UserProjectRole::admin()),
        "project_admin"
    );
}
