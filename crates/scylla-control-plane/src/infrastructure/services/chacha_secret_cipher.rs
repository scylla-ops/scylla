//! XChaCha20-Poly1305 AEAD adapter for [`SecretCipher`].
//!
//! Keyed by a 32-byte master key (64 hex chars) from the control-plane config.
//! Each ciphertext embeds a fresh random 24-byte nonce as `nonce || ciphertext`,
//! so encrypting any number of secrets under one key is safe. When no key is
//! configured the cipher is disabled and every operation errors with a clear
//! message rather than silently using a weak default.

use chacha20poly1305::aead::{Aead, Generate, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

use crate::application::secret::SecretCipher;
use crate::domain::errors::{DomainError, DomainResult};

const NONCE_LEN: usize = 24;

pub struct ChaChaSecretCipher {
    /// `None` when no master key was configured — every op then errors.
    cipher: Option<XChaCha20Poly1305>,
}

impl ChaChaSecretCipher {
    /// Build from an optional 64-hex-char (32-byte) master key. `None` yields a
    /// disabled cipher that errors on use.
    pub fn from_hex_key(master_key: Option<&str>) -> DomainResult<Self> {
        let cipher = match master_key {
            None => None,
            Some(hex) => {
                let key = decode_hex_32(hex)?;
                Some(XChaCha20Poly1305::new((&key).into()))
            }
        };
        Ok(Self { cipher })
    }

    fn active(&self) -> DomainResult<&XChaCha20Poly1305> {
        self.cipher.as_ref().ok_or_else(|| {
            DomainError::business_rule(
                "secret store is not configured; set [secrets] master_key in the control-plane config",
            )
        })
    }
}

impl SecretCipher for ChaChaSecretCipher {
    fn encrypt(&self, plaintext: &str) -> DomainResult<Vec<u8>> {
        let cipher = self.active()?;
        let nonce = XNonce::generate();
        let ct = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| DomainError::internal("secret encryption failed"))?;
        let mut blob = Vec::with_capacity(NONCE_LEN + ct.len());
        blob.extend_from_slice(nonce.as_slice());
        blob.extend_from_slice(&ct);
        Ok(blob)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> DomainResult<String> {
        let cipher = self.active()?;
        if ciphertext.len() < NONCE_LEN {
            return Err(DomainError::internal("secret ciphertext is too short"));
        }
        let (nonce_bytes, ct) = ciphertext.split_at(NONCE_LEN);
        // split_at guarantees NONCE_LEN bytes, so the conversion cannot fail.
        let nonce = XNonce::try_from(nonce_bytes)
            .map_err(|_| DomainError::internal("secret ciphertext is too short"))?;
        let pt = cipher
            .decrypt(&nonce, ct)
            .map_err(|_| DomainError::internal("secret decryption failed (wrong master key?)"))?;
        String::from_utf8(pt)
            .map_err(|_| DomainError::internal("decrypted secret is not valid UTF-8"))
    }
}

/// Decode a 64-char hex string into 32 bytes.
fn decode_hex_32(s: &str) -> DomainResult<[u8; 32]> {
    let s = s.trim();
    if s.len() != 64 {
        return Err(DomainError::validation(
            "secret master key must be 64 hex characters (32 bytes)",
        ));
    }
    let bytes = s.as_bytes();
    let mut out = [0u8; 32];
    for (i, pair) in bytes.chunks_exact(2).enumerate() {
        out[i] = (hex_val(pair[0])? << 4) | hex_val(pair[1])?;
    }
    Ok(out)
}

fn hex_val(c: u8) -> DomainResult<u8> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(DomainError::validation(
            "secret master key must be hexadecimal",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn round_trips() {
        let cipher = ChaChaSecretCipher::from_hex_key(Some(KEY)).unwrap();
        let blob = cipher.encrypt("hunter2").unwrap();
        assert_ne!(blob, b"hunter2");
        assert_eq!(cipher.decrypt(&blob).unwrap(), "hunter2");
    }

    #[test]
    fn distinct_nonces_per_encrypt() {
        let cipher = ChaChaSecretCipher::from_hex_key(Some(KEY)).unwrap();
        assert_ne!(
            cipher.encrypt("same").unwrap(),
            cipher.encrypt("same").unwrap(),
            "each encryption must use a fresh nonce"
        );
    }

    #[test]
    fn disabled_without_key() {
        let cipher = ChaChaSecretCipher::from_hex_key(None).unwrap();
        assert!(cipher.encrypt("x").is_err());
    }

    #[test]
    fn rejects_bad_key() {
        assert!(ChaChaSecretCipher::from_hex_key(Some("short")).is_err());
    }
}
