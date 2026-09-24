//! The secret's writes. One block per command, in the order it runs: the struct, its
//! permission, its payload types, what `Prepare` builds, what `Persist` writes.

use super::SecretUseCases;
use crate::application::SecretRepository;
use crate::domain::errors::DomainResult;
use crate::domain::ids::ProjectId;
use crate::domain::permission::Permission;
use crate::domain::secret::{Secret, SecretName};
use async_trait::async_trait;
use scylla_auth::authz::PermissionService;
use scylla_extension::{
    Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};

/// No `Debug`: `value` is the plaintext. `Prepare` encrypts it, so only the ciphertext is staged.
pub struct CreateSecret {
    pub project_id: ProjectId,
    pub name: SecretName,
    pub description: String,
    pub value: String,
}

impl Describe for CreateSecret {
    fn permission(&self) -> Permission {
        Permission::CreateSecret(self.project_id.clone())
    }
}

impl Command for CreateSecret {
    type Staged = Draft<Secret>;
    type Committed = Secret;
}

#[async_trait]
impl<R, PS> Run<Prepare<CreateSecret>> for SecretUseCases<R, PS>
where
    R: SecretRepository,
    PS: PermissionService,
{
    async fn run(&self, input: Authorized<CreateSecret>) -> DomainResult<Prepared<CreateSecret>> {
        let cmd = input.command();
        let encrypted = self.cipher.encrypt(&cmd.value)?;
        let secret = Secret::create(
            cmd.project_id.clone(),
            cmd.name.clone(),
            cmd.description.clone(),
            encrypted,
        );
        Ok(input.prepared(Draft::new(secret)))
    }
}

#[async_trait]
impl<R, PS> Run<Persist<CreateSecret>> for SecretUseCases<R, PS>
where
    R: SecretRepository,
    PS: PermissionService,
{
    async fn run(&self, input: Prepared<CreateSecret>) -> DomainResult<Committed<CreateSecret>> {
        input
            .commit(async |draft| {
                let secret = draft.into_inner();
                self.secret_repo.create(&secret).await?;
                Ok(secret)
            })
            .await
    }
}
