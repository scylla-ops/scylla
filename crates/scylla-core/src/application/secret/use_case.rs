use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::application::secret::SecretCipher;
use crate::application::secret::repository::SecretRepository;
use crate::domain::entities::{ProjectId, Secret, SecretId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::secret::SecretName;
use std::sync::Arc;
use tracing::instrument;

/// Project-scoped management of secrets. Every method is Cedar-gated. Values are
/// encrypted on create and never returned; only metadata is read back.
pub struct SecretUseCases<R, PS>
where
    R: SecretRepository,
    PS: PermissionService,
{
    secret_repo: Arc<R>,
    cipher: Arc<dyn SecretCipher>,
    permission_service: Arc<PS>,
}

impl<R, PS> SecretUseCases<R, PS>
where
    R: SecretRepository,
    PS: PermissionService,
{
    #[must_use]
    pub fn new(
        secret_repo: Arc<R>,
        cipher: Arc<dyn SecretCipher>,
        permission_service: Arc<PS>,
    ) -> Self {
        Self {
            secret_repo,
            cipher,
            permission_service,
        }
    }

    #[instrument(skip(self, caller, value), fields(project_id = %project_id, name = %name))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        project_id: ProjectId,
        name: SecretName,
        description: String,
        value: String,
    ) -> DomainResult<Secret> {
        self.permission_service
            .check(caller, Permission::CreateSecret(project_id.clone()))
            .await?;
        let encrypted = self.cipher.encrypt(&value)?;
        let secret = Secret::create(project_id, name, description, encrypted);
        self.secret_repo.create(&secret).await?;
        Ok(secret)
    }

    #[instrument(skip(self, caller), fields(project_id = %project_id))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        project_id: &ProjectId,
    ) -> DomainResult<Vec<Secret>> {
        self.permission_service
            .check(caller, Permission::ListSecrets(project_id.clone()))
            .await?;
        self.secret_repo.list_by_project(project_id).await
    }

    #[instrument(skip(self, caller), fields(secret_id = %secret_id))]
    pub async fn delete(&self, caller: &CallerContext, secret_id: &SecretId) -> DomainResult<()> {
        // Load first so we can authorize against the owning project.
        let secret = self.secret_repo.find_by_id(secret_id).await?;
        self.permission_service
            .check(caller, Permission::DeleteSecret(secret.project_id().clone()))
            .await?;
        self.secret_repo.delete(secret_id).await
    }
}
