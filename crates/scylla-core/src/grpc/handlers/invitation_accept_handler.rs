use crate::application::InvitationAcceptUseCases;
use crate::grpc::adapter::run_public;
use crate::grpc::convert::wrap;
use derive_more::Constructor;
use scylla_extension::Actions;
use scylla_proto::invitation::v1::{
    AcceptInvitationRequest, AcceptInvitationResponse,
    invitation_accept_service_server::InvitationAcceptService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct InvitationAcceptHandler {
    actions: Arc<Actions>,
    accepts: Arc<InvitationAcceptUseCases>,
}

#[async_trait::async_trait]
impl InvitationAcceptService for InvitationAcceptHandler {
    async fn accept_invitation(
        &self,
        request: Request<AcceptInvitationRequest>,
    ) -> Result<Response<AcceptInvitationResponse>, Status> {
        let outcome = run_public(&self.actions, &*self.accepts, request).await?;
        Ok(Response::new(AcceptInvitationResponse {
            token: outcome.token,
            user_id: wrap(outcome.user_id.to_string()),
            organization_id: wrap(outcome.organization_id.to_string()),
        }))
    }
}
