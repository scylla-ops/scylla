use crate::grpc::convert::{required, ts};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{
    AppCredentialRepository, AppRepository, AppTokenRepository, AppTokenUseCases, HashService,
};
use scylla_core::domain::entities::AppId;
use scylla_core::domain::value_objects::app::AppSecret;
use scylla_protocol::services::app::{
    IssueTokenRequest, IssueTokenResponse, app_auth_service_server::AppAuthService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AppAuthHandler<A, T, C, H>
where
    A: AppRepository,
    T: AppTokenRepository,
    C: AppCredentialRepository,
    H: HashService,
{
    use_cases: Arc<AppTokenUseCases<A, T, C, H>>,
}

#[async_trait::async_trait]
impl<
    A: AppRepository + Send + Sync + 'static,
    T: AppTokenRepository + Send + Sync + 'static,
    C: AppCredentialRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
> AppAuthService for AppAuthHandler<A, T, C, H>
{
    async fn issue_token(
        &self,
        request: Request<IssueTokenRequest>,
    ) -> Result<Response<IssueTokenResponse>, Status> {
        let req = request.into_inner();
        let app_id = AppId::new(&required(req.app_id, "app_id")?);
        // A malformed secret is treated as an auth failure, not a validation
        // error, so the response never reveals why credentials were rejected.
        let secret = AppSecret::new(&req.secret)
            .map_err(|_| Status::unauthenticated("Invalid app credentials"))?;

        let outcome = self
            .use_cases
            .issue(app_id, secret)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(IssueTokenResponse {
            token: outcome.token,
            expires_at: ts(outcome.expires_at),
        }))
    }
}
