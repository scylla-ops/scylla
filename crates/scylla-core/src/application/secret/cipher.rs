use crate::domain::errors::DomainResult;

pub trait SecretCipher: Send + Sync {
    fn encrypt(&self, plaintext: &str) -> DomainResult<Vec<u8>>;
    fn decrypt(&self, ciphertext: &[u8]) -> DomainResult<String>;
}
