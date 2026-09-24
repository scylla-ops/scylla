use crate::application::SignupUseCases;
use crate::grpc::convert::{required, wrap};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_domain::domain::organization::OrganizationName;
use scylla_domain::domain::user::{Email, Password, Username};
use scylla_proto::registration::v1::{
    SignupRequest, SignupResponse, registration_service_server::RegistrationService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct RegistrationHandler {
    signup_uc: Arc<SignupUseCases>,
}

#[async_trait::async_trait]
impl RegistrationService for RegistrationHandler {
    async fn signup(
        &self,
        request: Request<SignupRequest>,
    ) -> Result<Response<SignupResponse>, Status> {
        let req = request.into_inner();

        let username = Username::new(&req.username).map_err(domain_error_to_status)?;
        let email = Email::new(&required(req.email, "email")?).map_err(domain_error_to_status)?;
        let password = Password::new(&req.password).map_err(domain_error_to_status)?;
        let organization_name =
            OrganizationName::new(&req.organization_name).map_err(domain_error_to_status)?;

        let outcome = self
            .signup_uc
            .signup(username, email, password, organization_name)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(SignupResponse {
            token: outcome.token,
            user_id: wrap(outcome.user_id.to_string()),
            organization_id: wrap(outcome.organization_id.to_string()),
        }))
    }
}
