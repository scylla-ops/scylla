use crate::application::{InvitationRepository, InvitationUseCases, OrganizationRepository};
use crate::extract_auth_context;
use crate::grpc::adapter::run;
use crate::grpc::convert::id;
use crate::grpc::mappers::{domain_error_to_status, invitation_to_proto};
use derive_more::Constructor;
use scylla_auth::authz::PermissionService;
use scylla_domain::domain::ids::InvitationId;
use scylla_extension::Actions;
use scylla_proto::invitation::v1::{
    CreateInvitationRequest, CreateInvitationResponse, ListInvitationsRequest,
    ListInvitationsResponse, RevokeInvitationRequest, RevokeInvitationResponse,
    invitation_service_server::InvitationService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct InvitationHandler<I, O, PS>
where
    I: InvitationRepository,
    O: OrganizationRepository,
    PS: PermissionService,
{
    actions: Arc<Actions>,
    invitations: Arc<InvitationUseCases<I, O, PS>>,
}

#[async_trait::async_trait]
impl<
    I: InvitationRepository + 'static,
    O: OrganizationRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> InvitationService for InvitationHandler<I, O, PS>
{
    async fn create_invitation(
        &self,
        request: Request<CreateInvitationRequest>,
    ) -> Result<Response<CreateInvitationResponse>, Status> {
        let invitation = run(&self.actions, &*self.invitations, request).await?;
        Ok(Response::new(CreateInvitationResponse {
            invitation: Some(invitation_to_proto(&invitation)),
        }))
    }

    async fn list_invitations(
        &self,
        request: Request<ListInvitationsRequest>,
    ) -> Result<Response<ListInvitationsResponse>, Status> {
        let invitations = run(&self.actions, &*self.invitations, request).await?;
        Ok(Response::new(ListInvitationsResponse {
            invitations: invitations.iter().map(invitation_to_proto).collect(),
        }))
    }

    async fn revoke_invitation(
        &self,
        request: Request<RevokeInvitationRequest>,
    ) -> Result<Response<RevokeInvitationResponse>, Status> {
        let caller = caller!(request);
        let id: InvitationId = id(request.into_inner().invitation_id, "invitation_id")?;
        self.invitations
            .revoke(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(RevokeInvitationResponse {}))
    }
}
