use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::application::secret::SecretCipher;
use crate::application::secret::repository::SecretRepository;
use crate::domain::errors::DomainResult;
use crate::domain::ids::{ProjectId, SecretId};
use crate::domain::permission::Permission;
use crate::domain::secret::Secret;
use crate::domain::secret::SecretName;
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
            .check(
                caller,
                Permission::DeleteSecret(secret.project_id().clone()),
            )
            .await?;
        self.secret_repo.delete(secret_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::DomainError;
    use crate::domain::ids::UserId;
    use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService};
    use async_trait::async_trait;

    /// Secret repo + cipher that panic if touched: proves a denied call never
    /// reaches a side effect.
    struct ForbiddenRepo;
    #[async_trait]
    impl SecretRepository for ForbiddenRepo {
        async fn create(&self, _: &Secret) -> DomainResult<()> {
            panic!("secret_repo.create must not run when authorization is denied")
        }
        async fn find_by_id(&self, _: &SecretId) -> DomainResult<Secret> {
            panic!("secret_repo.find_by_id must not run when authorization is denied")
        }
        async fn list_by_project(&self, _: &ProjectId) -> DomainResult<Vec<Secret>> {
            panic!("secret_repo.list_by_project must not run when authorization is denied")
        }
        async fn delete(&self, _: &SecretId) -> DomainResult<()> {
            panic!("secret_repo.delete must not run when authorization is denied")
        }
    }
    struct ForbiddenCipher;
    impl SecretCipher for ForbiddenCipher {
        fn encrypt(&self, _: &str) -> DomainResult<Vec<u8>> {
            panic!("cipher.encrypt must not run when authorization is denied")
        }
        fn decrypt(&self, _: &[u8]) -> DomainResult<String> {
            unreachable!()
        }
    }

    /// A repo/cipher that accept writes, for the authorized happy path.
    struct OkRepo;
    #[async_trait]
    impl SecretRepository for OkRepo {
        async fn create(&self, _: &Secret) -> DomainResult<()> {
            Ok(())
        }
        async fn find_by_id(&self, _: &SecretId) -> DomainResult<Secret> {
            unimplemented!()
        }
        async fn list_by_project(&self, _: &ProjectId) -> DomainResult<Vec<Secret>> {
            unimplemented!()
        }
        async fn delete(&self, _: &SecretId) -> DomainResult<()> {
            unimplemented!()
        }
    }
    struct OkCipher;
    impl SecretCipher for OkCipher {
        fn encrypt(&self, _: &str) -> DomainResult<Vec<u8>> {
            Ok(vec![0xAA, 0xBB])
        }
        fn decrypt(&self, _: &[u8]) -> DomainResult<String> {
            unimplemented!()
        }
    }

    fn caller() -> CallerContext {
        CallerContext::User(UserId::new("u1"))
    }
    fn project() -> ProjectId {
        ProjectId::new("proj-1")
    }
    fn name() -> SecretName {
        SecretName::new("DB_PASSWORD").unwrap()
    }

    #[tokio::test]
    async fn create_authorizes_create_secret_on_the_target_project() {
        let perms = Arc::new(RecordingPermissionService::new());
        let uc = SecretUseCases::new(Arc::new(OkRepo), Arc::new(OkCipher), perms.clone());

        uc.create(&caller(), project(), name(), "desc".into(), "value".into())
            .await
            .unwrap();

        // The exact permission on the exact project must have been checked — a
        // copy-paste against the wrong project id would fail here.
        assert_eq!(
            perms.permissions(),
            vec![Permission::CreateSecret(project())]
        );
    }

    #[tokio::test]
    async fn create_denied_never_encrypts_or_persists() {
        let uc = SecretUseCases::new(
            Arc::new(ForbiddenRepo),
            Arc::new(ForbiddenCipher),
            Arc::new(DenyingPermissionService::new()),
        );

        let err = uc
            .create(&caller(), project(), name(), "desc".into(), "value".into())
            .await
            .expect_err("a denied create must not succeed");

        assert!(
            matches!(err, DomainError::Forbidden(_)),
            "denial must surface as Forbidden, got {err:?}",
        );
        // The panicking doubles prove authorization ran before any side effect.
    }
}
