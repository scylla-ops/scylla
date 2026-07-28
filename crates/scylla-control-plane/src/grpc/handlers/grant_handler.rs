use crate::extract_auth_context;
use crate::grpc::convert::{
    principal_ref_from_proto, principal_ref_to_proto, required, scope_kind_from_proto,
    scope_kind_to_proto, scope_ref_from_proto, scope_ref_to_proto, wrap,
};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use crate::application::{
    Grant, GrantRepository, GrantUseCases, GrantableRole, PermissionService, PolicyControl,
    RoleKind, grantable_roles,
};
use scylla_core::domain::value_objects::role::RoleName;
use scylla_protocol::authz::v1::{
    CreateGrantRequest, CreateGrantResponse, Grant as ProtoGrant,
    GrantableRole as ProtoGrantableRole, ListGrantableRolesRequest, ListGrantableRolesResponse,
    ListGrantsRequest, ListGrantsResponse, RevokeAllAccessRequest, RevokeAllAccessResponse,
    RevokeGrantRequest, RevokeGrantResponse, RoleKind as ProtoRoleKind,
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
    ) -> Result<Response<CreateGrantResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        // A grant is about a user *or* a machine app — the `PrincipalRef` union
        // carries the kind with its id, so neither is inferred here.
        let principal = principal_ref_from_proto(req.principal)?;
        let scope = scope_ref_from_proto(req.scope)?;

        let role = RoleName::new(required(req.role, "role")?).map_err(domain_error_to_status)?;
        let grant = Grant::new(principal, role, scope);
        self.use_cases
            .grant(&caller, &grant)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(CreateGrantResponse {
            grant: Some(grant_to_proto(&grant)),
        }))
    }

    async fn revoke_grant(
        &self,
        request: Request<RevokeGrantRequest>,
    ) -> Result<Response<RevokeGrantResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let grant_id = required(req.grant_id, "grant_id")?;

        self.use_cases
            .revoke(&caller, &grant_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RevokeGrantResponse {}))
    }

    async fn revoke_all_access(
        &self,
        request: Request<RevokeAllAccessRequest>,
    ) -> Result<Response<RevokeAllAccessResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let principal = principal_ref_from_proto(req.principal)?;
        let scope = scope_ref_from_proto(req.scope)?;

        let revoked = self
            .use_cases
            .revoke_all_access(&caller, &principal, &scope)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RevokeAllAccessResponse { revoked }))
    }

    async fn list_grants(
        &self,
        request: Request<ListGrantsRequest>,
    ) -> Result<Response<ListGrantsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        // Scope filter present → scoped listing (org/project/system admins of it).
        // Absent → list every grant (system admins only). The bound entity travels
        // inside the `ScopeRef` arm, so SYSTEM carries no id at all.
        let grants = match req.scope {
            Some(scope) => {
                let scope = scope_ref_from_proto(Some(scope))?;
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
        let filter = req.scope_kind.map(scope_kind_from_proto).transpose()?;

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
        // Builtin role ids are their stable keys, so the catalog name is the id.
        role_id: wrap(r.name),
        scope_kind: scope_kind_to_proto(r.scope) as i32,
        kind: role_kind_to_proto(r.kind) as i32,
        description: r.description.to_string(),
    }
}

/// Domain `RoleKind` → the proto enum. Both arms are real kinds, so
/// `UNSPECIFIED` is never produced.
fn role_kind_to_proto(kind: RoleKind) -> ProtoRoleKind {
    match kind {
        RoleKind::Admin => ProtoRoleKind::Admin,
        RoleKind::Member => ProtoRoleKind::Member,
        RoleKind::Agent => ProtoRoleKind::Agent,
    }
}

fn grant_to_proto(g: &Grant) -> ProtoGrant {
    ProtoGrant {
        grant_id: wrap(g.id.clone()),
        principal: Some(principal_ref_to_proto(&g.principal)),
        scope: Some(scope_ref_to_proto(&g.scope)),
        role: wrap(g.role.to_string()),
    }
}
