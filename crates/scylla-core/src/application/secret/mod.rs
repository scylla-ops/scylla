pub mod cipher;
pub mod commands;
pub mod queries;
pub mod repository;
pub mod resolver;

pub use cipher::SecretCipher;
pub use commands::CreateSecret;
pub use queries::ListSecrets;
pub use repository::SecretRepository;
pub use resolver::{DispatchSecretResolver, SecretResolver};

use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::ids::SecretId;
use crate::domain::permission::Permission;
use derive_more::Constructor;
use scylla_auth::authz::PermissionService;
use std::sync::Arc;
use tracing::instrument;

/// The secret aggregate's stage runners, one block per action in `commands.rs` and
/// `queries.rs`. `delete` stays outside the pipeline: its permission is on the secret's
/// project, which only the loaded secret knows, and `Describe` sees the command alone.
#[derive(Constructor)]
pub struct SecretUseCases {
    pub(super) secret_repo: Arc<dyn SecretRepository>,
    pub(super) cipher: Arc<dyn SecretCipher>,
    pub(super) permission_service: Arc<dyn PermissionService>,
}

impl SecretUseCases {
    #[instrument(skip_all, fields(secret_id = %secret_id))]
    pub async fn delete(&self, caller: &CallerContext, secret_id: &SecretId) -> DomainResult<()> {
        let secret = self.secret_repo.find_by_id(secret_id).await?;
        self.permission_service
            .check(
                caller,
                Permission::DeleteSecret(secret.project_id().clone()),
            )
            .await?;
        self.secret_repo.delete(secret_id).await
    }
}

#[cfg(test)]
mod tests;
