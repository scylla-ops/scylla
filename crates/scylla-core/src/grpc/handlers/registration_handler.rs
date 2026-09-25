use crate::application::SignupUseCases;
use crate::grpc::adapter::run_public;
use crate::grpc::convert::wrap;
use derive_more::Constructor;
use scylla_extension::Actions;
use scylla_proto::registration::v1::{
    SignupRequest, SignupResponse, registration_service_server::RegistrationService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct RegistrationHandler {
    actions: Arc<Actions>,
    signups: Arc<SignupUseCases>,
}

#[async_trait::async_trait]
impl RegistrationService for RegistrationHandler {
    async fn signup(
        &self,
        request: Request<SignupRequest>,
    ) -> Result<Response<SignupResponse>, Status> {
        let outcome = run_public(&self.actions, &*self.signups, request).await?;
        Ok(Response::new(SignupResponse {
            token: outcome.token,
            user_id: wrap(outcome.user_id.to_string()),
            organization_id: wrap(outcome.organization_id.to_string()),
        }))
    }
}
