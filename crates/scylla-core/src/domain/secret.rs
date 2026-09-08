mod name;

pub use name::*;

use crate::domain::clock;
use crate::domain::ids::{ProjectId, SecretId};
use chrono::{DateTime, Utc};

/// A project-scoped secret: a named, encrypted value referenced from pipeline
/// node env vars and decrypted only at dispatch. The entity carries the
/// **ciphertext**, never the plaintext; the API never returns either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secret {
    id: SecretId,
    project_id: ProjectId,
    name: SecretName,
    description: String,
    /// AEAD ciphertext blob (nonce embedded). Storage-only.
    encrypted_value: Vec<u8>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Secret {
    /// Create a new secret from an already-encrypted value.
    #[must_use]
    pub fn create(
        project_id: ProjectId,
        name: SecretName,
        description: String,
        encrypted_value: Vec<u8>,
    ) -> Self {
        let now = clock::now();
        Self {
            id: SecretId::generate(),
            project_id,
            name,
            description,
            encrypted_value,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitute from persistence.
    #[must_use]
    pub fn from_persistence(
        id: SecretId,
        project_id: ProjectId,
        name: SecretName,
        description: String,
        encrypted_value: Vec<u8>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            description,
            encrypted_value,
            created_at,
            updated_at,
        }
    }

    #[must_use]
    pub fn id(&self) -> &SecretId {
        &self.id
    }

    #[must_use]
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    #[must_use]
    pub fn name(&self) -> &SecretName {
        &self.name
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub fn encrypted_value(&self) -> &[u8] {
        &self.encrypted_value
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
