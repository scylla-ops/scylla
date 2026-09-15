use crate::application::SecretCipher;
use crate::application::trigger::delivery::TriggerDeliveryRepository;
use crate::application::trigger::fire::TriggerFiring;
use crate::application::trigger::repository::TriggerRepository;
use crate::domain::clock;
use crate::domain::errors::DomainError;
use crate::domain::ids::{JobId, TriggerId};
use crate::domain::trigger::TriggerSource;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tracing::instrument;

pub const DEFAULT_SIGNATURE_HEADER: &str = "X-Scylla-Signature-256";

#[derive(Debug)]
pub enum IngestOutcome {
    Fired(JobId),
    Duplicate,
}

#[derive(Debug)]
pub enum WebhookError {
    /// Opaque 404 so the endpoint never reveals which trigger ids exist.
    NotFound,
    BadSignature,
    Internal(DomainError),
}

/// Signature verification happens before any write, so an unauthenticated caller cannot pollute the dedupe table.
pub struct WebhookIngressUseCases<T, D>
where
    T: TriggerRepository,
    D: TriggerDeliveryRepository,
{
    trigger_repo: Arc<T>,
    delivery_repo: Arc<D>,
    cipher: Arc<dyn SecretCipher>,
    firing: Arc<dyn TriggerFiring>,
}

impl<T, D> WebhookIngressUseCases<T, D>
where
    T: TriggerRepository,
    D: TriggerDeliveryRepository,
{
    #[must_use]
    pub fn new(
        trigger_repo: Arc<T>,
        delivery_repo: Arc<D>,
        cipher: Arc<dyn SecretCipher>,
        firing: Arc<dyn TriggerFiring>,
    ) -> Self {
        Self {
            trigger_repo,
            delivery_repo,
            cipher,
            firing,
        }
    }

    #[instrument(skip_all, fields(trigger_id = %trigger_id))]
    pub async fn ingest(
        &self,
        trigger_id: &TriggerId,
        get_header: &(dyn for<'a> Fn(&'a str) -> Option<String> + Sync),
        delivery_id: Option<&str>,
        raw_body: &[u8],
    ) -> Result<IngestOutcome, WebhookError> {
        let trigger = match self.trigger_repo.find_by_id(trigger_id).await {
            Ok(t) => t,
            Err(e) if e.is_not_found() => return Err(WebhookError::NotFound),
            Err(e) => return Err(WebhookError::Internal(e)),
        };
        if !trigger.is_enabled() {
            return Err(WebhookError::NotFound);
        }
        let TriggerSource::Webhook(spec) = trigger.source() else {
            return Err(WebhookError::NotFound);
        };
        let header_name = spec
            .signature_header()
            .unwrap_or(DEFAULT_SIGNATURE_HEADER)
            .to_string();

        let secret = match self.trigger_repo.webhook_secret(trigger_id).await {
            Ok(Some(enc)) => self.cipher.decrypt(&enc).map_err(WebhookError::Internal)?,
            Ok(None) => {
                return Err(WebhookError::Internal(DomainError::internal(
                    "webhook trigger has no signing secret",
                )));
            }
            Err(e) => return Err(WebhookError::Internal(e)),
        };

        let Some(signature) = get_header(&header_name) else {
            return Err(WebhookError::BadSignature);
        };
        if !verify_signature(&secret, raw_body, &signature) {
            return Err(WebhookError::BadSignature);
        }

        let key = delivery_id
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&signature);
        let is_new = self
            .delivery_repo
            .record_or_detect(trigger_id, key, clock::now())
            .await
            .map_err(WebhookError::Internal)?;
        if !is_new {
            return Ok(IngestOutcome::Duplicate);
        }

        let payload: Option<serde_json::Value> = serde_json::from_slice(raw_body).ok();
        let job = self
            .firing
            .fire(trigger_id, payload.as_ref(), delivery_id)
            .await
            .map_err(WebhookError::Internal)?;
        Ok(IngestOutcome::Fired(job.id().clone()))
    }
}

#[must_use]
pub fn verify_signature(secret: &str, raw_body: &[u8], signature: &str) -> bool {
    let expected = signature
        .strip_prefix("sha256=")
        .unwrap_or(signature)
        .trim();
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(raw_body);
    let computed = hex::encode(mac.finalize().into_bytes());
    // ct_eq is length-aware: no early return on length mismatch.
    bool::from(computed.as_bytes().ct_eq(expected.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "key";
    const MSG: &[u8] = b"The quick brown fox jumps over the lazy dog";
    const SIG: &str = "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8";

    #[test]
    fn accepts_correct_signature_with_and_without_prefix() {
        assert!(verify_signature(KEY, MSG, SIG));
        assert!(verify_signature(KEY, MSG, &format!("sha256={SIG}")));
    }

    #[test]
    fn rejects_tampered_body_wrong_key_and_garbage() {
        assert!(!verify_signature(KEY, b"tampered", SIG));
        assert!(!verify_signature("wrong-key", MSG, SIG));
        assert!(!verify_signature(KEY, MSG, "not-hex"));
        assert!(!verify_signature(KEY, MSG, ""));
    }
}
