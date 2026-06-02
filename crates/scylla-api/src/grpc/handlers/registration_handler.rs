use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::authz::policy::PolicyControl;
use scylla_core::application::{HashService, SessionRepository, SignupRepository, SignupUseCases};
use scylla_core::domain::value_objects::organization::OrganizationName;
use scylla_core::domain::value_objects::user::{Email, Password, Username};
use scylla_protocol::services::registration::{
    SignupRequest, SignupResponse, registration_service_server::RegistrationService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// Public self-service signup. Lives behind the `signup` cargo feature so PaaS
/// builds don't register it at all (admin-provisioned accounts only).
#[derive(Constructor)]
pub struct RegistrationHandler<SR, S, H, PC>
where
    SR: SignupRepository,
    S: SessionRepository,
    H: HashService,
    PC: PolicyControl,
{
    signup_uc: Arc<SignupUseCases<SR, S, H, PC>>,
}

#[async_trait::async_trait]
impl<
    SR: SignupRepository + Send + Sync + 'static,
    S: SessionRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> RegistrationService for RegistrationHandler<SR, S, H, PC>
{
    async fn signup(
        &self,
        request: Request<SignupRequest>,
    ) -> Result<Response<SignupResponse>, Status> {
        let req = request.into_inner();

        let username = Username::new(&req.username).map_err(domain_error_to_status)?;
        let email = Email::new(&req.email).map_err(domain_error_to_status)?;
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
            user_id: outcome.user_id.to_string(),
            organization_id: outcome.organization_id.to_string(),
        }))
    }
}
