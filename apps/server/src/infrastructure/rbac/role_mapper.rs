pub struct RoleMapper;

impl RoleMapper {
    /// Map global role to Casbin role name
    pub fn global_role_to_casbin(
        role: &crate::domain::value_objects::UserGlobalRole,
    ) -> &'static str {
        match role.as_str() {
            "admin" => "admin",
            "user" => "user",
            _ => "user",
        }
    }

    /// Map organization role to Casbin role name
    pub fn org_role_to_casbin(
        role: &crate::domain::value_objects::UserOrganizationRole,
    ) -> &'static str {
        match role.as_str() {
            "owner" => "org_owner",
            "admin" => "org_admin",
            "member" => "org_member",
            _ => "org_member",
        }
    }

    /// Map project role to Casbin role name
    pub fn project_role_to_casbin(
        role: &crate::domain::value_objects::UserProjectRole,
    ) -> &'static str {
        match role.as_str() {
            "owner" => "project_owner",
            "admin" => "project_admin",
            "member" => "project_member",
            _ => "project_member",
        }
    }
}
