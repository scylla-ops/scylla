use crate::extract_auth_context;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{PermissionService, UserRoleRepository, UserRoleUseCases};
use scylla_core::domain::entities::UserId;
use scylla_core::domain::value_objects::role::name::RoleName;
use scylla_protocol::services::permission::{
    AssignRoleRequest, AssignRoleResponse, ListUserRolesRequest, ListUserRolesResponse,
    RevokeRoleRequest, RevokeRoleResponse, role_service_server::RoleService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct RoleHandler<URR: UserRoleRepository, PS: PermissionService> {
    use_cases: Arc<UserRoleUseCases<URR, PS>>,
}

#[async_trait::async_trait]
impl<
    URR: UserRoleRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> RoleService for RoleHandler<URR, PS>
{
    async fn assign_role(
        &self,
        request: Request<AssignRoleRequest>,
    ) -> Result<Response<AssignRoleResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let role = RoleName::new(&req.role).map_err(domain_error_to_status)?;

        self.use_cases
            .assign(&caller, &user_id, &role)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AssignRoleResponse {}))
    }

    async fn revoke_role(
        &self,
        request: Request<RevokeRoleRequest>,
    ) -> Result<Response<RevokeRoleResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);
        let role = RoleName::new(&req.role).map_err(domain_error_to_status)?;

        self.use_cases
            .revoke(&caller, &user_id, &role)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RevokeRoleResponse {}))
    }

    async fn list_user_roles(
        &self,
        request: Request<ListUserRolesRequest>,
    ) -> Result<Response<ListUserRolesResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);

        let roles = self
            .use_cases
            .list_roles(&caller, &user_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ListUserRolesResponse {
            roles: roles.iter().map(ToString::to_string).collect(),
        }))
    }
}
