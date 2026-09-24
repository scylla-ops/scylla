use crate::application::InvitationAcceptUseCases;
use crate::grpc::convert::wrap;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_domain::domain::user::{Password, Username};
use scylla_proto::invitation::v1::{
    AcceptInvitationRequest, AcceptInvitationResponse,
    invitation_accept_service_server::InvitationAcceptService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct InvitationAcceptHandler {
    use_cases: Arc<InvitationAcceptUseCases>,
}

#[async_trait::async_trait]
impl InvitationAcceptService for InvitationAcceptHandler {
    async fn accept_invitation(
        &self,
        request: Request<AcceptInvitationRequest>,
    ) -> Result<Response<AcceptInvitationResponse>, Status> {
        let req = request.into_inner();
        let username = Username::new(&req.username).map_err(domain_error_to_status)?;
        let password = Password::new(&req.password).map_err(domain_error_to_status)?;

        let outcome = self
            .use_cases
            .accept(&req.token, username, password)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(AcceptInvitationResponse {
            token: outcome.token,
            user_id: wrap(outcome.user_id.to_string()),
            organization_id: wrap(outcome.organization_id.to_string()),
        }))
    }
}
