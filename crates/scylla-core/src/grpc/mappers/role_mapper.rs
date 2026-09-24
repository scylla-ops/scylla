//! Wire to command, command outcome to wire. The handler holds none of it.

use crate::application::role::{
    CreateRole, DeleteRole, GetEffectivePermissions, GetRole, ListAuthzVocabulary, ListRoles,
    UpdateRole,
};
use crate::grpc::convert::{
    Parse, permission_from_key, permission_key, principal_ref_from_proto, required,
    scope_kind_from_proto, scope_kind_to_proto, scope_ref_to_proto, wrap,
};
use scylla_auth::authz::{EffectiveScope, FULL_CONTROL, Role, resource_home_scope};
use scylla_proto::authz::v1::{
    Access, AuthzAction, CreateRoleRequest, DeleteRoleRequest,
    EffectiveScope as ProtoEffectiveScope, GetEffectivePermissionsRequest, GetRoleRequest,
    ListAuthzVocabularyRequest, ListRolesRequest, Permission, Role as ProtoRole, UpdateRoleRequest,
    access, role,
};
use tonic::Status;

pub fn role_to_proto(role: &Role) -> ProtoRole {
    let origin = match &role.key {
        Some(key) => role::Origin::Builtin(role::Builtin { key: key.clone() }),
        None => role::Origin::Custom(role::Custom {
            owner_organization_id: role.owner_org.as_ref().and_then(|id| wrap(id.to_string())),
        }),
    };
    ProtoRole {
        role_id: wrap(role.id.clone()),
        name: role.name.clone(),
        description: role.description.clone(),
        scope_kind: scope_kind_to_proto(role.scope) as i32,
        access: Some(access_from_keys(role.is_full_control(), &role.permissions)),
        origin: Some(origin),
    }
}

pub fn effective_scope_to_proto(es: &EffectiveScope) -> ProtoEffectiveScope {
    ProtoEffectiveScope {
        scope: Some(scope_ref_to_proto(&es.scope)),
        access: Some(access_from_keys(es.full_control, &es.permissions)),
    }
}

/// `resource_type` is derivable from the permission; only `min_scope` ships.
pub fn authz_action_to_proto((key, resource_type): &(&str, &str)) -> AuthzAction {
    AuthzAction {
        permission: permission_from_key(key).map_or(0, |p| p as i32),
        min_scope: scope_kind_to_proto(resource_home_scope(resource_type)) as i32,
    }
}

fn access_from_keys(full_control: bool, keys: &[String]) -> Access {
    let inner = if full_control {
        access::Access::FullControl(access::FullControl {})
    } else {
        access::Access::Restricted(access::Restricted {
            permissions: keys
                .iter()
                .filter_map(|key| permission_from_key(key).map(|p| p as i32))
                .collect(),
        })
    };
    Access {
        access: Some(inner),
    }
}

fn permissions_from_proto(access: Option<Access>) -> Result<Vec<String>, Status> {
    let access = access.ok_or_else(|| Status::invalid_argument("access is required"))?;
    match access.access {
        Some(access::Access::FullControl(_)) => Ok(vec![FULL_CONTROL.to_string()]),
        Some(access::Access::Restricted(r)) => r
            .permissions
            .iter()
            .map(|&p| {
                let perm = Permission::try_from(p)
                    .map_err(|_| Status::invalid_argument("unknown permission value"))?;
                permission_key(perm)
                    .ok_or_else(|| Status::invalid_argument("permission unspecified"))
            })
            .collect(),
        None => Err(Status::invalid_argument(
            "access is required (full_control or restricted)",
        )),
    }
}

impl Parse for CreateRoleRequest {
    type Into = CreateRole;

    fn parse(self) -> Result<CreateRole, Status> {
        Ok(CreateRole {
            scope: scope_kind_from_proto(self.scope_kind)?,
            permissions: permissions_from_proto(self.access)?,
            name: self.name,
            description: self.description,
        })
    }
}

impl Parse for UpdateRoleRequest {
    type Into = UpdateRole;

    fn parse(self) -> Result<UpdateRole, Status> {
        Ok(UpdateRole {
            id: required(self.role_id, "role_id")?,
            permissions: permissions_from_proto(self.access)?,
            name: self.name,
            description: self.description,
        })
    }
}

impl Parse for DeleteRoleRequest {
    type Into = DeleteRole;

    fn parse(self) -> Result<DeleteRole, Status> {
        Ok(DeleteRole {
            id: required(self.role_id, "role_id")?,
        })
    }
}

impl Parse for ListRolesRequest {
    type Into = ListRoles;

    fn parse(self) -> Result<ListRoles, Status> {
        Ok(ListRoles)
    }
}

impl Parse for GetRoleRequest {
    type Into = GetRole;

    fn parse(self) -> Result<GetRole, Status> {
        Ok(GetRole {
            id: required(self.role_id, "role_id")?,
        })
    }
}

impl Parse for GetEffectivePermissionsRequest {
    type Into = GetEffectivePermissions;

    fn parse(self) -> Result<GetEffectivePermissions, Status> {
        Ok(GetEffectivePermissions {
            principal: principal_ref_from_proto(self.principal)?,
        })
    }
}

impl Parse for ListAuthzVocabularyRequest {
    type Into = ListAuthzVocabulary;

    fn parse(self) -> Result<ListAuthzVocabulary, Status> {
        Ok(ListAuthzVocabulary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scylla_auth::authz::ScopeKind;
    use scylla_proto::authz::v1::ScopeKind as ProtoScopeKind;
    use tonic::Code;

    fn restricted(permissions: &[Permission]) -> Option<Access> {
        Some(Access {
            access: Some(access::Access::Restricted(access::Restricted {
                permissions: permissions.iter().map(|p| *p as i32).collect(),
            })),
        })
    }

    #[test]
    fn a_create_request_becomes_a_command_with_permission_keys() {
        let command = CreateRoleRequest {
            name: "CI".to_string(),
            description: "runs the builds".to_string(),
            scope_kind: ProtoScopeKind::Project as i32,
            access: restricted(&[Permission::ReadPipeline, Permission::RunPipeline]),
        }
        .parse()
        .unwrap();

        assert_eq!(command.name, "CI");
        assert_eq!(command.scope, ScopeKind::Project);
        assert_eq!(
            command.permissions,
            vec!["readPipeline".to_string(), "runPipeline".to_string()]
        );
    }

    #[test]
    fn full_control_becomes_the_wildcard() {
        let command = CreateRoleRequest {
            name: "Admin".to_string(),
            description: String::new(),
            scope_kind: ProtoScopeKind::Organization as i32,
            access: Some(Access {
                access: Some(access::Access::FullControl(access::FullControl {})),
            }),
        }
        .parse()
        .unwrap();

        assert_eq!(command.permissions, vec![FULL_CONTROL.to_string()]);
    }

    #[test]
    fn an_unspecified_scope_kind_is_an_invalid_argument() {
        let Err(err) = CreateRoleRequest {
            name: "CI".to_string(),
            description: String::new(),
            scope_kind: ProtoScopeKind::Unspecified as i32,
            access: restricted(&[Permission::ReadPipeline]),
        }
        .parse() else {
            panic!("an unspecified scope kind must be refused");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn an_update_without_access_is_an_invalid_argument() {
        let Err(err) = UpdateRoleRequest {
            role_id: wrap("ci"),
            name: "CI".to_string(),
            description: String::new(),
            access: None,
        }
        .parse() else {
            panic!("a missing access must be refused");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn a_get_request_needs_a_role_id() {
        assert_eq!(
            GetRoleRequest {
                role_id: wrap("ci")
            }
            .parse()
            .unwrap()
            .id,
            "ci"
        );
        let Err(err) = GetRoleRequest { role_id: None }.parse() else {
            panic!("a missing role id must be refused");
        };
        assert_eq!(err.code(), Code::InvalidArgument);
    }
}
