use crate::extract_auth_context;
use crate::grpc::convert::{
    permission_from_key, permission_key, required, scope_kind_from_proto, scope_kind_to_proto, wrap,
};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    Grant, GrantRepository, GrantTarget, GrantUseCases, GrantableRole, PermissionService,
    PolicyControl, Principal, Scope, grantable_roles,
};
use scylla_core::domain::entities::{OrganizationId, ProjectId, UserId};
use scylla_core::domain::value_objects::role::RoleName;
use scylla_protocol::services::permission::{
    CreateGrantRequest, Grant as ProtoGrant, GrantableRole as ProtoGrantableRole,
    ListGrantableRolesRequest, ListGrantableRolesResponse, ListGrantsRequest, ListGrantsResponse,
    Permission, RevokeGrantRequest, RevokeGrantResponse, Scope as ProtoScope,
    grant_service_server::GrantService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct GrantHandler<G: GrantRepository, PC: PolicyControl, PS: PermissionService> {
    use_cases: Arc<GrantUseCases<G, PC, PS>>,
}

#[async_trait::async_trait]
impl<
    G: GrantRepository + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> GrantService for GrantHandler<G, PC, PS>
{
    async fn create_grant(
        &self,
        request: Request<CreateGrantRequest>,
    ) -> Result<Response<ProtoGrant>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&required(req.user_id, "user_id")?);
        let scope = scope_from_proto(req.scope, &req.scope_id)?;

        // A `permission` set → grant that single permission (additive); otherwise
        // grant the named `role`. Both go through the same anti-escalation check.
        let grant = match req.permission {
            Some(p) if p != Permission::Unspecified as i32 => {
                let perm = Permission::try_from(p)
                    .map_err(|_| Status::invalid_argument("unknown permission value"))?;
                let key = permission_key(perm)
                    .ok_or_else(|| Status::invalid_argument("permission unspecified"))?;
                Grant::with_permission(Principal::User(user_id), key, scope)
            }
            _ => {
                let role = RoleName::new(&req.role).map_err(domain_error_to_status)?;
                Grant::new(Principal::User(user_id), role, scope)
            }
        };
        self.use_cases
            .grant(&caller, &grant)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(grant_to_proto(&grant)))
    }

    async fn revoke_grant(
        &self,
        request: Request<RevokeGrantRequest>,
    ) -> Result<Response<RevokeGrantResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        self.use_cases
            .revoke(&caller, &req.id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RevokeGrantResponse { revoked: true }))
    }

    async fn list_grants(
        &self,
        request: Request<ListGrantsRequest>,
    ) -> Result<Response<ListGrantsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        // Scope filter present → scoped listing (org/project/system admins of it).
        // Absent → list every grant (system admins only). The `scope_id` is
        // ignored for SYSTEM (singleton root) and used for org/project.
        let grants = match req.scope {
            Some(kind) => {
                let scope = scope_from_proto(kind, req.scope_id.as_deref().unwrap_or(""))?;
                self.use_cases.list_by_scope(&caller, &scope).await
            }
            None => self.use_cases.list(&caller).await,
        }
        .map_err(domain_error_to_status)?;

        Ok(Response::new(ListGrantsResponse {
            grants: grants.iter().map(grant_to_proto).collect(),
        }))
    }

    async fn list_grantable_roles(
        &self,
        request: Request<ListGrantableRolesRequest>,
    ) -> Result<Response<ListGrantableRolesResponse>, Status> {
        // Authenticated callers only (interceptor); the catalog itself is static,
        // non-sensitive compile-time data, so no Cedar check is applied.
        let _caller = caller!(request);
        let req = request.into_inner();
        let filter = req.scope.map(scope_kind_from_proto).transpose()?;

        Ok(Response::new(ListGrantableRolesResponse {
            roles: grantable_roles(filter)
                .iter()
                .map(grantable_role_to_proto)
                .collect(),
        }))
    }
}

fn grantable_role_to_proto(r: &GrantableRole) -> ProtoGrantableRole {
    ProtoGrantableRole {
        name: r.name.to_string(),
        scope: scope_kind_to_proto(r.scope) as i32,
        kind: r.kind.as_str().to_string(),
        description: r.description.to_string(),
    }
}

fn scope_from_proto(kind: i32, id: &str) -> Result<Scope, Status> {
    match ProtoScope::try_from(kind) {
        // System is the tenancy root; scope_id is ignored (single root).
        Ok(ProtoScope::System) => Ok(Scope::System),
        Ok(ProtoScope::Organization) => Ok(Scope::Organization(OrganizationId::new(id))),
        Ok(ProtoScope::Project) => Ok(Scope::Project(ProjectId::new(id))),
        Ok(ProtoScope::Unspecified) | Err(_) => {
            Err(Status::invalid_argument("unknown or unspecified scope"))
        }
    }
}

fn grant_to_proto(g: &Grant) -> ProtoGrant {
    let (scope, scope_id) = match &g.scope {
        Scope::System => (ProtoScope::System, String::new()),
        Scope::Organization(id) => (ProtoScope::Organization, id.to_string()),
        Scope::Project(id) => (ProtoScope::Project, id.to_string()),
    };
    // A role grant fills `role`; a permission grant fills `permission`.
    let (role, permission) = match &g.target {
        GrantTarget::Role(r) => (r.to_string(), None),
        GrantTarget::Permission(key) => (String::new(), permission_from_key(key).map(|p| p as i32)),
    };
    ProtoGrant {
        id: g.id.clone(),
        user_id: wrap(g.principal.id().to_string()),
        role,
        scope: scope as i32,
        scope_id,
        permission,
    }
}
