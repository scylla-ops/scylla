use crate::application::{AccountOutcome, OAuthOutcome, OAuthUseCases};
use crate::grpc::convert::wrap;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_proto::oauth::v1::{
    CallbackRequest, CallbackResponse, GetAuthUrlRequest, GetAuthUrlResponse, callback_response,
    callback_response::{ExistingAccount, NewAccount},
    oauth_service_server::OauthService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct OAuthHandler {
    use_cases: Arc<OAuthUseCases>,
}

#[async_trait::async_trait]
impl OauthService for OAuthHandler {
    async fn get_auth_url(
        &self,
        request: Request<GetAuthUrlRequest>,
    ) -> Result<Response<GetAuthUrlResponse>, Status> {
        let req = request.into_inner();
        let url = self
            .use_cases
            .authorize_url(&req.state)
            .map_err(domain_error_to_status)?;
        Ok(Response::new(GetAuthUrlResponse { url }))
    }

    async fn callback(
        &self,
        request: Request<CallbackRequest>,
    ) -> Result<Response<CallbackResponse>, Status> {
        let req = request.into_inner();
        let OAuthOutcome {
            token,
            user_id,
            account,
        } = self
            .use_cases
            .callback(&req.code)
            .await
            .map_err(domain_error_to_status)?;
        let outcome = match account {
            AccountOutcome::New { organization_id } => {
                callback_response::Outcome::NewAccount(NewAccount {
                    organization_id: wrap(organization_id.to_string()),
                })
            }
            AccountOutcome::Existing => {
                callback_response::Outcome::ExistingAccount(ExistingAccount {})
            }
        };
        Ok(Response::new(CallbackResponse {
            token,
            user_id: wrap(user_id.to_string()),
            outcome: Some(outcome),
        }))
    }
}
