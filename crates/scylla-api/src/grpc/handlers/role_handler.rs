use crate::extract_auth_context;
use crate::grpc::convert::{
    permission_from_key, permission_key, scope_kind_from_proto, scope_kind_to_proto,
};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    FULL_CONTROL, GrantRepository, PermissionService, PolicyControl, Role, RoleRepository,
    RoleUseCases,
};
use scylla_protocol::services::permission::{
    CreateRoleRequest, DeleteRoleRequest, DeleteRoleResponse, GetRoleRequest, ListRolesRequest,
    ListRolesResponse, Permission, Role as ProtoRole, UpdateRoleRequest,
    role_service_server::RoleService,
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
        let permissions = permissions_from_proto(req.full_control, &req.permissions)?;

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
        let permissions = permissions_from_proto(req.full_control, &req.permissions)?;

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
    let full_control = role.is_full_control();
    let permissions = if full_control {
        Vec::new()
    } else {
        role.permissions
            .iter()
            .filter_map(|key| permission_from_key(key).map(|p| p as i32))
            .collect()
    };
    ProtoRole {
        id: role.id.clone(),
        key: role.key.clone(),
        name: role.name.clone(),
        description: role.description.clone(),
        scope: scope_kind_to_proto(role.scope) as i32,
        builtin: role.builtin,
        full_control,
        permissions,
    }
}
