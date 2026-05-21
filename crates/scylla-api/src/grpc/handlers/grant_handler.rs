use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    Grant, GrantPrincipal, GrantRepository, GrantScope, GrantUseCases, PermissionService,
    PolicyControl,
};
use scylla_core::domain::entities::{OrganizationId, ProjectId, UserId};
use scylla_core::domain::value_objects::role::name::RoleName;
use scylla_protocol::services::permission::{
    CreateGrantRequest, Grant as ProtoGrant, GrantScopeKind, ListGrantsRequest, ListGrantsResponse,
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

        // Scope filter present → scoped listing (org/project admins). Absent →
        // list every grant (system admins only).
        let grants = match (req.scope_kind, req.scope_id) {
            (Some(kind), Some(id)) => {
                let scope = scope_from_proto(kind, &id)?;
                self.use_cases.list_by_scope(&caller, &scope).await
            }
            _ => self.use_cases.list(&caller).await,
        }
        .map_err(domain_error_to_status)?;

        Ok(Response::new(ListGrantsResponse {
            grants: grants.iter().map(grant_to_proto).collect(),
        }))
    }
}

fn scope_from_proto(kind: i32, id: &str) -> Result<GrantScope, Status> {
    match GrantScopeKind::try_from(kind) {
        Ok(GrantScopeKind::GrantScopeOrganization) => {
            Ok(GrantScope::Organization(OrganizationId::new(id)))
        }
        Ok(GrantScopeKind::GrantScopeProject) => Ok(GrantScope::Project(ProjectId::new(id))),
        Err(_) => Err(Status::invalid_argument("unknown grant scope kind")),
    }
}

fn grant_to_proto(g: &Grant) -> ProtoGrant {
    let (scope_kind, scope_id) = match &g.scope {
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
