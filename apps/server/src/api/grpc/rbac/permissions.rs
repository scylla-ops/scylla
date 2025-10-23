// permission constants and helper functions for RBAC

/// role constants
pub mod roles {
    pub const OWNER: &str = "owner";
    pub const ADMIN: &str = "admin";
    pub const MEMBER: &str = "member";
    pub const VIEWER: &str = "viewer";
}

/// action constants
pub mod actions {
    pub const READ: &str = "read";
    pub const WRITE: &str = "write";
    pub const DELETE: &str = "delete";
    pub const MANAGE_USERS: &str = "manage_users";
    pub const EXECUTE: &str = "execute";
    pub const CANCEL: &str = "cancel";
}

/// resource type constants
pub mod resources {
    pub const ORGANIZATIONS: &str = "organizations";
    pub const PROJECTS: &str = "projects";
    #[allow(dead_code)]
    pub const PIPELINES: &str = "pipelines";
    #[allow(dead_code)]
    pub const JOBS: &str = "jobs";
}

/// defines which actions each role can perform
pub fn role_permissions(role: &str) -> Vec<&'static str> {
    match role {
        roles::OWNER => vec![
            actions::READ,
            actions::WRITE,
            actions::DELETE,
            actions::MANAGE_USERS,
            actions::EXECUTE,
            actions::CANCEL,
        ],
        roles::ADMIN => vec![
            actions::READ,
            actions::WRITE,
            actions::MANAGE_USERS,
            actions::EXECUTE,
            actions::CANCEL,
        ],
        roles::MEMBER => vec![
            actions::READ,
            actions::WRITE,
            actions::EXECUTE,
            actions::CANCEL,
        ],
        roles::VIEWER => vec![actions::READ],
        _ => vec![],
    }
}

/// helper to create policy rules for a role in a domain
#[allow(dead_code)]
pub fn create_role_policies(
    user_id: &str,
    domain_id: &str,
    resource: &str,
    role: &str,
) -> Vec<Vec<String>> {
    let permissions = role_permissions(role);
    permissions
        .into_iter()
        .map(|action| {
            vec![
                user_id.to_string(),
                domain_id.to_string(),
                resource.to_string(),
                action.to_string(),
            ]
        })
        .collect()
}

