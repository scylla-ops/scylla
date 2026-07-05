use crate::extract_auth_context;
use crate::grpc::convert::{
    permission_from_key, permission_key, scope_kind_from_proto, scope_kind_to_proto, scope_to_proto,
};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    EffectiveScope, FULL_CONTROL, GrantRepository, PermissionService, PolicyControl, Principal,
    Role, RoleRepository, RoleUseCases,
};
use scylla_core::domain::entities::{AppId, UserId};
use scylla_protocol::services::permission::{
    CreateRoleRequest, DeleteRoleRequest, DeleteRoleResponse,
    EffectiveScope as ProtoEffectiveScope, FullControl, GetEffectivePermissionsRequest,
    GetEffectivePermissionsResponse, GetRoleRequest, ListRolesRequest, ListRolesResponse,
    Permission, PrincipalKind, RestrictedPermissions, Role as ProtoRole, UpdateRoleRequest,
    create_role_request, effective_scope, role, role_service_server::RoleService,
    update_role_request,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct RoleHandler<RR, GR, PS, PC>
where
    RR: RoleRepository,
    GR: GrantRepository,
    PS: PermissionService,
    PC: PolicyControl,
{
    use_cases: Arc<RoleUseCases<RR, GR, PS, PC>>,
}

#[async_trait::async_trait]
impl<
    RR: RoleRepository + Send + Sync + 'static,
    GR: GrantRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> RoleService for RoleHandler<RR, GR, PS, PC>
{
    async fn create_role(
        &self,
        request: Request<CreateRoleRequest>,
    ) -> Result<Response<ProtoRole>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let scope = scope_kind_from_proto(req.scope)?;
        let (full_control, permission_ids) = match req.access {
            Some(create_role_request::Access::FullControl(_)) => (true, Vec::new()),
            Some(create_role_request::Access::Restricted(r)) => (false, r.permissions),
            None => {
                return Err(Status::invalid_argument(
                    "access is required (full_control or restricted)",
                ));
            }
        };
        let permissions = permissions_from_proto(full_control, &permission_ids)?;

        let role = self
            .use_cases
            .create(&caller, req.name, req.description, scope, permissions)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(role_to_proto(&role)))
    }

    async fn update_role(
        &self,
        request: Request<UpdateRoleRequest>,
    ) -> Result<Response<ProtoRole>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let (full_control, permission_ids) = match req.access {
            Some(update_role_request::Access::FullControl(_)) => (true, Vec::new()),
            Some(update_role_request::Access::Restricted(r)) => (false, r.permissions),
            None => {
                return Err(Status::invalid_argument(
                    "access is required (full_control or restricted)",
                ));
            }
        };
        let permissions = permissions_from_proto(full_control, &permission_ids)?;

        let role = self
            .use_cases
            .update(&caller, &req.id, req.name, req.description, permissions)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(role_to_proto(&role)))
    }

    async fn delete_role(
        &self,
        request: Request<DeleteRoleRequest>,
    ) -> Result<Response<DeleteRoleResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        self.use_cases
            .delete(&caller, &req.id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteRoleResponse { deleted: true }))
    }

    async fn list_roles(
        &self,
        request: Request<ListRolesRequest>,
    ) -> Result<Response<ListRolesResponse>, Status> {
        let caller = caller!(request);

        let roles = self
            .use_cases
            .list(&caller)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ListRolesResponse {
            roles: roles.iter().map(role_to_proto).collect(),
        }))
    }

    async fn get_role(
        &self,
        request: Request<GetRoleRequest>,
    ) -> Result<Response<ProtoRole>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        let role = self
            .use_cases
            .get(&caller, &req.id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(role_to_proto(&role)))
    }

    async fn get_effective_permissions(
        &self,
        request: Request<GetEffectivePermissionsRequest>,
    ) -> Result<Response<GetEffectivePermissionsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let principal = principal_from_proto(req.principal_kind, req.principal_id)?;

        let scopes = self
            .use_cases
            .effective_permissions(&caller, &principal)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(GetEffectivePermissionsResponse {
            scopes: scopes.iter().map(effective_scope_to_proto).collect(),
        }))
    }
}

fn principal_from_proto(kind: i32, id: String) -> Result<Principal, Status> {
    match PrincipalKind::try_from(kind) {
        Ok(PrincipalKind::User) => Ok(Principal::User(UserId::new(id))),
        Ok(PrincipalKind::App) => Ok(Principal::App(AppId::new(id))),
        Ok(PrincipalKind::Unspecified) | Err(_) => Err(Status::invalid_argument(
            "unknown or unspecified principal kind",
        )),
    }
}

fn effective_scope_to_proto(es: &EffectiveScope) -> ProtoEffectiveScope {
    let (scope, scope_id) = scope_to_proto(&es.scope);
    let access = if es.full_control {
        effective_scope::Access::FullControl(FullControl {})
    } else {
        effective_scope::Access::Restricted(restricted_from_keys(&es.permissions))
    };
    ProtoEffectiveScope {
        scope: scope as i32,
        scope_id,
        access: Some(access),
    }
}

/// The proto `RestrictedPermissions` for a set of domain permission keys.
fn restricted_from_keys(keys: &[String]) -> RestrictedPermissions {
    RestrictedPermissions {
        permissions: keys
            .iter()
            .filter_map(|key| permission_from_key(key).map(|p| p as i32))
            .collect(),
    }
}

/// Build the domain permission set from the proto: `full_control` → the `*`
/// sentinel; otherwise the named permission keys.
fn permissions_from_proto(full_control: bool, permissions: &[i32]) -> Result<Vec<String>, Status> {
    if full_control {
        return Ok(vec![FULL_CONTROL.to_string()]);
    }
    permissions
        .iter()
        .map(|&p| {
            let perm = Permission::try_from(p)
                .map_err(|_| Status::invalid_argument("unknown permission value"))?;
            permission_key(perm).ok_or_else(|| Status::invalid_argument("permission unspecified"))
        })
        .collect()
}

fn role_to_proto(role: &Role) -> ProtoRole {
    let access = if role.is_full_control() {
        role::Access::FullControl(FullControl {})
    } else {
        role::Access::Restricted(restricted_from_keys(&role.permissions))
    };
    ProtoRole {
        id: role.id.clone(),
        key: role.key.clone(),
        name: role.name.clone(),
        description: role.description.clone(),
        scope: scope_kind_to_proto(role.scope) as i32,
        builtin: role.builtin,
        access: Some(access),
    }
}
