use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    Grant, GrantPrincipal, GrantRepository, GrantScope, GrantUseCases, GrantableRole,
    PermissionService, PolicyControl, ScopeKind, grantable_roles,
};
use scylla_core::domain::entities::{OrganizationId, ProjectId, UserId};
use scylla_core::domain::value_objects::role::name::RoleName;
use scylla_protocol::services::permission::{
    CreateGrantRequest, Grant as ProtoGrant, GrantScopeKind, GrantableRole as ProtoGrantableRole,
    ListGrantableRolesRequest, ListGrantableRolesResponse, ListGrantsRequest, ListGrantsResponse,
    RevokeGrantRequest, RevokeGrantResponse, grant_service_server::GrantService,
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
        let user_id = UserId::new(&req.user_id);
        let role = RoleName::new(&req.role).map_err(domain_error_to_status)?;
        let scope = scope_from_proto(req.scope_kind, &req.scope_id)?;

        let grant = Grant::new(GrantPrincipal::User(user_id), role, scope);
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
        let grants = match req.scope_kind {
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
        let filter = req.scope_kind.map(scope_kind_from_proto).transpose()?;

        Ok(Response::new(ListGrantableRolesResponse {
            roles: grantable_roles(filter)
                .iter()
                .map(grantable_role_to_proto)
                .collect(),
        }))
    }
}

/// Map a proto `GrantScopeKind` discriminant to the id-free domain `ScopeKind`
/// (used to filter the assignable-role catalog).
fn scope_kind_from_proto(kind: i32) -> Result<ScopeKind, Status> {
    match GrantScopeKind::try_from(kind) {
        Ok(GrantScopeKind::GrantScopeSystem) => Ok(ScopeKind::System),
        Ok(GrantScopeKind::GrantScopeOrganization) => Ok(ScopeKind::Organization),
        Ok(GrantScopeKind::GrantScopeProject) => Ok(ScopeKind::Project),
        Err(_) => Err(Status::invalid_argument("unknown grant scope kind")),
    }
}

fn scope_kind_to_proto(kind: ScopeKind) -> GrantScopeKind {
    match kind {
        ScopeKind::System => GrantScopeKind::GrantScopeSystem,
        ScopeKind::Organization => GrantScopeKind::GrantScopeOrganization,
        ScopeKind::Project => GrantScopeKind::GrantScopeProject,
    }
}

fn grantable_role_to_proto(r: &GrantableRole) -> ProtoGrantableRole {
    ProtoGrantableRole {
        name: r.name.to_string(),
        scope_kind: scope_kind_to_proto(r.scope) as i32,
        kind: r.kind.as_str().to_string(),
        description: r.description.to_string(),
    }
}

fn scope_from_proto(kind: i32, id: &str) -> Result<GrantScope, Status> {
    match GrantScopeKind::try_from(kind) {
        // System is the tenancy root; scope_id is ignored (single root).
        Ok(GrantScopeKind::GrantScopeSystem) => Ok(GrantScope::System),
        Ok(GrantScopeKind::GrantScopeOrganization) => {
            Ok(GrantScope::Organization(OrganizationId::new(id)))
        }
        Ok(GrantScopeKind::GrantScopeProject) => Ok(GrantScope::Project(ProjectId::new(id))),
        Err(_) => Err(Status::invalid_argument("unknown grant scope kind")),
    }
}

fn grant_to_proto(g: &Grant) -> ProtoGrant {
    let (scope_kind, scope_id) = match &g.scope {
        GrantScope::System => (GrantScopeKind::GrantScopeSystem, String::new()),
        GrantScope::Organization(id) => (GrantScopeKind::GrantScopeOrganization, id.to_string()),
        GrantScope::Project(id) => (GrantScopeKind::GrantScopeProject, id.to_string()),
    };
    ProtoGrant {
        id: g.id.clone(),
        user_id: g.principal.id().to_string(),
        role: g.role.to_string(),
        scope_kind: scope_kind as i32,
        scope_id,
    }
}
