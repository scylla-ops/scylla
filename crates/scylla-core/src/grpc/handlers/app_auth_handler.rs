use crate::application::AppTokenUseCases;
use crate::grpc::convert::{required, ts};
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_domain::domain::app::AppSecret;
use scylla_domain::domain::ids::AppId;
use scylla_proto::app::v1::{
    IssueTokenRequest, IssueTokenResponse, app_auth_service_server::AppAuthService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AppAuthHandler {
    use_cases: Arc<AppTokenUseCases>,
}

#[async_trait::async_trait]
impl AppAuthService for AppAuthHandler {
    async fn issue_token(
        &self,
        request: Request<IssueTokenRequest>,
    ) -> Result<Response<IssueTokenResponse>, Status> {
        let req = request.into_inner();
        let app_id = AppId::new(&required(req.app_id, "app_id")?);
        // A malformed secret is an auth failure, so the response never says why.
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
