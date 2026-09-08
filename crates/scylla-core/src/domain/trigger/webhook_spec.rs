use crate::domain::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

/// Configuration of a webhook source.
///
/// `signature_header` is the request header carrying the hex HMAC-SHA256 of the
/// raw body; `None` selects the Scylla default header. The signing secret itself
/// is NOT stored here — it is generated at create time and kept in the encrypted
/// (AEAD) secret store (HMAC verification needs the plaintext, so a one-way hash
/// would be useless). That storage is wired in the webhook-ingress work; this VO
/// stays forward-compatible via serde defaults. Serialized as part of the tagged
/// [`super::TriggerSource`] JSONB blob.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WebhookSpec {
    #[serde(default)]
    signature_header: Option<String>,
}

impl WebhookSpec {
    /// Build a webhook spec. A provided signature header must be non-empty
    /// (trimmed); `None` falls back to the Scylla default header.
    pub fn new(signature_header: Option<String>) -> DomainResult<Self> {
        let signature_header = match signature_header {
            Some(h) => {
                let trimmed = h.trim();
                if trimmed.is_empty() {
                    return Err(DomainError::validation(
                        "Signature header, when set, cannot be empty",
                    ));
                }
                Some(trimmed.to_string())
            }
            None => None,
        };
        Ok(Self { signature_header })
    }

    /// The configured signature header, or `None` to use the Scylla default.
    #[must_use]
    pub fn signature_header(&self) -> Option<&str> {
        self.signature_header.as_deref()
    }
}
