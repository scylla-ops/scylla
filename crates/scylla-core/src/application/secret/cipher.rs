use crate::domain::errors::DomainResult;

/// Reversible encryption for project secret values. Implemented by an
/// infrastructure AEAD adapter keyed by the control-plane master key. The
/// ciphertext blob is opaque (it embeds the nonce); store it verbatim.
pub trait SecretCipher: Send + Sync {
    /// Encrypt a plaintext secret value into an opaque ciphertext blob.
    fn encrypt(&self, plaintext: &str) -> DomainResult<Vec<u8>>;
    /// Decrypt a ciphertext blob produced by [`Self::encrypt`].
    fn decrypt(&self, ciphertext: &[u8]) -> DomainResult<String>;
}
