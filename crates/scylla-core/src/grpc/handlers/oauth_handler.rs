use crate::application::OAuthUseCases;
use crate::grpc::adapter::run_public;
use derive_more::Constructor;
use scylla_extension::Actions;
use scylla_proto::oauth::v1::{
    CallbackRequest, CallbackResponse, GetAuthUrlRequest, GetAuthUrlResponse,
    oauth_service_server::OauthService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct OAuthHandler {
    actions: Arc<Actions>,
    oauth: Arc<OAuthUseCases>,
}

#[async_trait::async_trait]
impl OauthService for OAuthHandler {
    async fn get_auth_url(
        &self,
        request: Request<GetAuthUrlRequest>,
    ) -> Result<Response<GetAuthUrlResponse>, Status> {
        let url = run_public(&self.actions, &*self.oauth, request).await?;
        Ok(Response::new(GetAuthUrlResponse { url }))
    }

    async fn callback(
        &self,
        request: Request<CallbackRequest>,
    ) -> Result<Response<CallbackResponse>, Status> {
        let outcome = run_public(&self.actions, &*self.oauth, request).await?;
        Ok(Response::new(outcome.into()))
    }
}
