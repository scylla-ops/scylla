use crate::domain::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

/// The signing secret lives in the AEAD secret store: HMAC needs the plaintext.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WebhookSpec {
    #[serde(default)]
    signature_header: Option<String>,
}

impl WebhookSpec {
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

    #[must_use]
    pub fn signature_header(&self) -> Option<&str> {
        self.signature_header.as_deref()
    }
}
