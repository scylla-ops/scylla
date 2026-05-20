use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::permission::policy::PolicyControl;
use scylla_core::application::{
    HashService, OAuthIdentityRepository, OAuthProvider, OAuthUseCases, SessionRepository,
    SignupRepository, UserRepository,
};
use scylla_protocol::services::oauth::{
    GetAuthUrlRequest, GetAuthUrlResponse, OauthCallbackRequest, OauthCallbackResponse,
    oauth_service_server::OauthService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// Public GitHub OAuth endpoints. Behind the `oauth-github` feature.
#[derive(Constructor)]
pub struct OAuthHandler<P, IR, SR, U, S, H, PC>
where
    P: OAuthProvider,
    IR: OAuthIdentityRepository,
    SR: SignupRepository,
    U: UserRepository,
    S: SessionRepository,
    H: HashService,
    PC: PolicyControl,
{
    use_cases: Arc<OAuthUseCases<P, IR, SR, U, S, H, PC>>,
}

#[async_trait::async_trait]
impl<
    P: OAuthProvider + Send + Sync + 'static,
    IR: OAuthIdentityRepository + Send + Sync + 'static,
    SR: SignupRepository + Send + Sync + 'static,
    U: UserRepository + Send + Sync + 'static,
    S: SessionRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
> OauthService for OAuthHandler<P, IR, SR, U, S, H, PC>
{
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
        request: Request<OauthCallbackRequest>,
    ) -> Result<Response<OauthCallbackResponse>, Status> {
        let req = request.into_inner();
        let outcome = self
            .use_cases
            .callback(&req.code)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(OauthCallbackResponse {
            token: outcome.token,
            user_id: outcome.user_id.to_string(),
            organization_id: outcome.organization_id.map(|o| o.to_string()),
        }))
    }
}
