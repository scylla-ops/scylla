use crate::application::agent::dispatch::{DispatchEnv, DispatchNode};
use crate::application::secret::SecretCipher;
use crate::application::secret::repository::SecretRepository;
use crate::domain::entities::{PipelineNode, ProjectId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::pipeline::EnvSource;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Turns a pipeline's definition nodes into resolved [`DispatchNode`]s: literal
/// env vars pass through; secret references are looked up in the pipeline's
/// project and decrypted, marked `masked` so the agent scrubs them from logs.
#[async_trait]
pub trait SecretResolver: Send + Sync {
    async fn resolve(
        &self,
        project_id: &ProjectId,
        nodes: &[PipelineNode],
    ) -> DomainResult<Vec<DispatchNode>>;
}

/// Default resolver backed by the secret repository + cipher.
pub struct DispatchSecretResolver<R>
where
    R: SecretRepository,
{
    secret_repo: Arc<R>,
    cipher: Arc<dyn SecretCipher>,
}

impl<R> DispatchSecretResolver<R>
where
    R: SecretRepository,
{
    #[must_use]
    pub fn new(secret_repo: Arc<R>, cipher: Arc<dyn SecretCipher>) -> Self {
        Self {
            secret_repo,
            cipher,
        }
    }
}

#[async_trait]
impl<R> SecretResolver for DispatchSecretResolver<R>
where
    R: SecretRepository,
{
    async fn resolve(
        &self,
        project_id: &ProjectId,
        nodes: &[PipelineNode],
    ) -> DomainResult<Vec<DispatchNode>> {
        // Only hit the secret store if at least one node references a secret.
        let needs_secrets = nodes
            .iter()
            .flat_map(PipelineNode::env)
            .any(|e| matches!(e.source(), EnvSource::Secret(_)));

        let by_name: HashMap<String, Vec<u8>> = if needs_secrets {
            self.secret_repo
                .list_by_project(project_id)
                .await?
                .into_iter()
                .map(|s| (s.name().as_str().to_string(), s.encrypted_value().to_vec()))
                .collect()
        } else {
            HashMap::new()
        };

        let mut out = Vec::with_capacity(nodes.len());
        for node in nodes {
            let mut env = Vec::with_capacity(node.env().len());
            for ev in node.env() {
                match ev.source() {
                    EnvSource::Literal(value) => env.push(DispatchEnv {
                        key: ev.key().to_string(),
                        value: value.clone(),
                        masked: false,
                    }),
                    EnvSource::Secret(name) => {
                        let ciphertext = by_name.get(name.as_str()).ok_or_else(|| {
                            DomainError::not_found("secret", name.as_str())
                        })?;
                        let value = self.cipher.decrypt(ciphertext)?;
                        env.push(DispatchEnv {
                            key: ev.key().to_string(),
                            value,
                            masked: true,
                        });
                    }
                }
            }
            out.push(DispatchNode {
                id: node.id().to_string(),
                deps: node.deps().iter().map(ToString::to_string).collect(),
                working_dir: node.working_dir().map(|w| w.as_str().to_string()),
                step: node.step().clone(),
                env,
            });
        }
        Ok(out)
    }
}
