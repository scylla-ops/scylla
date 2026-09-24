use crate::application::{
    HashService, InvitationAcceptUseCases, InvitationRepository, SessionRepository, UserRepository,
};
use crate::grpc::convert::wrap;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_auth::authz::PolicyControl;
use scylla_domain::domain::user::{Password, Username};
use scylla_proto::invitation::v1::{
    AcceptInvitationRequest, AcceptInvitationResponse,
    invitation_accept_service_server::InvitationAcceptService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct InvitationAcceptHandler<I, U, H, S, PC>
where
    I: InvitationRepository,
    U: UserRepository,
    H: HashService,
    S: SessionRepository,
    PC: PolicyControl,
{
    use_cases: Arc<InvitationAcceptUseCases<I, U, H, S, PC>>,
}

#[async_trait::async_trait]
impl<
    I: InvitationRepository + 'static,
    U: UserRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    S: SessionRepository + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> InvitationAcceptService for InvitationAcceptHandler<I, U, H, S, PC>
{
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
