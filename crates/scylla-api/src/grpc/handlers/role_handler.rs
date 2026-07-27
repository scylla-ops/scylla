use crate::extract_auth_context;
use crate::grpc::convert::{
    permission_from_key, permission_key, principal_ref_from_proto, required, scope_kind_from_proto,
    scope_kind_to_proto, scope_ref_to_proto, wrap,
};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    EffectiveScope, FULL_CONTROL, GrantRepository, PermissionService, PolicyControl, Role,
    RoleRepository, RoleUseCases, resource_home_scope,
};
use scylla_protocol::authz::v1::{
    Access, AuthzAction, CreateRoleRequest, CreateRoleResponse, DeleteRoleRequest,
    DeleteRoleResponse, EffectiveScope as ProtoEffectiveScope, GetEffectivePermissionsRequest,
    GetEffectivePermissionsResponse, GetMyPermissionsRequest, GetMyPermissionsResponse,
    GetRoleRequest, GetRoleResponse, ListAuthzVocabularyRequest, ListAuthzVocabularyResponse,
    ListRolesRequest, ListRolesResponse, Permission, Role as ProtoRole, UpdateRoleRequest,
    UpdateRoleResponse, access, role, role_service_server::RoleService,
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
    ) -> Result<Response<CreateRoleResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let scope = scope_kind_from_proto(req.scope_kind)?;
        let permissions = permissions_from_proto(req.access)?;

        let role = self
            .use_cases
            .create(&caller, req.name, req.description, scope, permissions)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(CreateRoleResponse {
            role: Some(role_to_proto(&role)),
        }))
    }

    async fn update_role(
        &self,
        request: Request<UpdateRoleRequest>,
    ) -> Result<Response<UpdateRoleResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let role_id = required(req.role_id, "role_id")?;
        let permissions = permissions_from_proto(req.access)?;

        let role = self
            .use_cases
            .update(&caller, &role_id, req.name, req.description, permissions)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(UpdateRoleResponse {
            role: Some(role_to_proto(&role)),
        }))
    }

    async fn delete_role(
        &self,
        request: Request<DeleteRoleRequest>,
    ) -> Result<Response<DeleteRoleResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let role_id = required(req.role_id, "role_id")?;

        self.use_cases
            .delete(&caller, &role_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteRoleResponse {}))
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
    ) -> Result<Response<GetRoleResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let role_id = required(req.role_id, "role_id")?;

        let role = self
            .use_cases
            .get(&caller, &role_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(GetRoleResponse {
            role: Some(role_to_proto(&role)),
        }))
    }

    async fn get_effective_permissions(
        &self,
        request: Request<GetEffectivePermissionsRequest>,
    ) -> Result<Response<GetEffectivePermissionsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let principal = principal_ref_from_proto(req.principal)?;

        let scopes = self
            .use_cases
            .effective_permissions(&caller, &principal)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(GetEffectivePermissionsResponse {
            scopes: scopes.iter().map(effective_scope_to_proto).collect(),
        }))
    }

    async fn get_my_permissions(
        &self,
        request: Request<GetMyPermissionsRequest>,
    ) -> Result<Response<GetMyPermissionsResponse>, Status> {
        let caller = caller!(request);

        let scopes = self
            .use_cases
            .my_permissions(&caller)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(GetMyPermissionsResponse {
            scopes: scopes.iter().map(effective_scope_to_proto).collect(),
        }))
    }

    async fn list_authz_vocabulary(
        &self,
        request: Request<ListAuthzVocabularyRequest>,
    ) -> Result<Response<ListAuthzVocabularyResponse>, Status> {
        let caller = caller!(request);

        let actions = self
            .use_cases
            .authz_vocabulary(&caller)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ListAuthzVocabularyResponse {
            // resource_type is derivable from the permission, so it never ships;
            // we keep min_scope (the one derived fact the client consumes) and
            // compute it server-side from the permission's resource type.
            actions: actions
                .iter()
                .map(|(key, resource_type)| AuthzAction {
                    permission: permission_from_key(key).map_or(0, |p| p as i32),
                    min_scope: scope_kind_to_proto(resource_home_scope(resource_type)) as i32,
                })
                .collect(),
        }))
    }
}

fn effective_scope_to_proto(es: &EffectiveScope) -> ProtoEffectiveScope {
    ProtoEffectiveScope {
        scope: Some(scope_ref_to_proto(&es.scope)),
        access: Some(access_from_keys(es.full_control, &es.permissions)),
    }
}

/// The shared proto `Access` for a domain permission set: full control carries
/// no list, anything else is the named permission keys.
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

/// Build the domain permission set from the proto `Access`: full control → the
/// `*` sentinel; otherwise the named permission keys. An absent oneof is a
/// client error, never a silent empty set.
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

fn role_to_proto(role: &Role) -> ProtoRole {
    // A builtin has a stable key and no owner; a custom role has an owner and no
    // key. The oneof makes the mixed case unrepresentable.
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
