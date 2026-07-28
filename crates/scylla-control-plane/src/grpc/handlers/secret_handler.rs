use crate::extract_auth_context;
use crate::grpc::convert::required;
use crate::grpc::mappers::{domain_error_to_status, secret_to_proto};
use crate::application::{PermissionService, SecretRepository, SecretUseCases};
use scylla_core::domain::entities::{ProjectId, SecretId};
use scylla_core::domain::value_objects::secret::SecretName;
use scylla_protocol::secret::v1::{
    CreateSecretRequest, CreateSecretResponse, DeleteSecretRequest, DeleteSecretResponse,
    ListSecretsRequest, ListSecretsResponse, secret_service_server::SecretService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct SecretHandler<R: SecretRepository, PS: PermissionService> {
    use_cases: Arc<SecretUseCases<R, PS>>,
}

impl<R: SecretRepository, PS: PermissionService> SecretHandler<R, PS> {
    pub fn new(use_cases: Arc<SecretUseCases<R, PS>>) -> Self {
        Self { use_cases }
    }
}

#[async_trait::async_trait]
impl<R: SecretRepository + Send + Sync + 'static, PS: PermissionService + Send + Sync + 'static>
    SecretService for SecretHandler<R, PS>
{
    async fn create_secret(
        &self,
        request: Request<CreateSecretRequest>,
    ) -> Result<Response<CreateSecretResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let project_id = ProjectId::new(&required(req.project_id, "project_id")?);
        let name = SecretName::new(&req.name).map_err(domain_error_to_status)?;
        let secret = self
            .use_cases
            .create(&caller, project_id, name, req.description, req.value)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(CreateSecretResponse {
            secret: Some(secret_to_proto(&secret)),
        }))
    }

    async fn list_secrets(
        &self,
        request: Request<ListSecretsRequest>,
    ) -> Result<Response<ListSecretsResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let project_id = ProjectId::new(&required(req.project_id, "project_id")?);
        let secrets = self
            .use_cases
            .list(&caller, &project_id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(ListSecretsResponse {
            secrets: secrets.iter().map(secret_to_proto).collect(),
        }))
    }

    async fn delete_secret(
        &self,
        request: Request<DeleteSecretRequest>,
    ) -> Result<Response<DeleteSecretResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = SecretId::new(&required(req.secret_id, "secret_id")?);
        self.use_cases
            .delete(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteSecretResponse {}))
    }
}
