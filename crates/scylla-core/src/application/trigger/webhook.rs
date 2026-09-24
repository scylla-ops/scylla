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
pub const PING_EVENT: &str = "ping";

#[derive(Debug)]
pub enum IngestOutcome {
    Fired(JobId),
    Duplicate,
    Ping,
}

#[derive(Debug)]
pub enum WebhookError {
    /// Opaque 404 so the endpoint never reveals which trigger ids exist.
    NotFound,
    BadSignature,
    Internal(DomainError),
}

/// Signature verification happens before any write, so an unauthenticated caller cannot pollute the dedupe table.
pub struct WebhookIngressUseCases {
    trigger_repo: Arc<dyn TriggerRepository>,
    delivery_repo: Arc<dyn TriggerDeliveryRepository>,
    cipher: Arc<dyn SecretCipher>,
    firing: Arc<dyn TriggerFiring>,
}

impl WebhookIngressUseCases {
    #[must_use]
    pub fn new(
        trigger_repo: Arc<dyn TriggerRepository>,
        delivery_repo: Arc<dyn TriggerDeliveryRepository>,
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
        event: Option<&str>,
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
        // Verified before the ping check: a wrong secret fails on GitHub's creation-time ping, not on the first push.
        if event.is_some_and(|e| e.trim().eq_ignore_ascii_case(PING_EVENT)) {
            return Ok(IngestOutcome::Ping);
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
    use crate::domain::errors::DomainResult;
    use crate::domain::ids::PipelineId;
    use crate::domain::job::Job;
    use crate::domain::trigger::{Trigger, TriggerName, WebhookSpec};
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use std::sync::Mutex;

    const KEY: &str = "key";
    const MSG: &[u8] = b"The quick brown fox jumps over the lazy dog";
    const SIG: &str = "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8";

    struct StubRepo {
        trigger: Trigger,
    }

    #[async_trait]
    impl TriggerRepository for StubRepo {
        async fn find_by_id(&self, _: &TriggerId) -> DomainResult<Trigger> {
            Ok(self.trigger.clone())
        }
        async fn webhook_secret(&self, _: &TriggerId) -> DomainResult<Option<Vec<u8>>> {
            Ok(Some(KEY.as_bytes().to_vec()))
        }
        async fn create(&self, _: &Trigger, _: Option<&[u8]>) -> DomainResult<Trigger> {
            unimplemented!()
        }
        async fn update(&self, _: &Trigger) -> DomainResult<Trigger> {
            unimplemented!()
        }
        async fn delete(&self, _: &TriggerId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_by_pipeline(&self, _: &PipelineId) -> DomainResult<Vec<Trigger>> {
            unimplemented!()
        }
        async fn list_unscheduled_cron(&self) -> DomainResult<Vec<Trigger>> {
            unimplemented!()
        }
        async fn claim_due_cron(
            &self,
            _: DateTime<Utc>,
            _: i64,
            _: &(dyn for<'a> Fn(&'a Trigger) -> DomainResult<DateTime<Utc>> + Sync),
        ) -> DomainResult<Vec<Trigger>> {
            unimplemented!()
        }
    }

    struct StubDeliveries {
        recorded: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl TriggerDeliveryRepository for StubDeliveries {
        async fn record_or_detect(
            &self,
            _: &TriggerId,
            delivery_id: &str,
            _: DateTime<Utc>,
        ) -> DomainResult<bool> {
            let mut seen = self.recorded.lock().unwrap();
            let is_new = !seen.iter().any(|d| d == delivery_id);
            seen.push(delivery_id.to_string());
            Ok(is_new)
        }
    }

    struct PlainCipher;

    impl SecretCipher for PlainCipher {
        fn encrypt(&self, plaintext: &str) -> DomainResult<Vec<u8>> {
            Ok(plaintext.as_bytes().to_vec())
        }
        fn decrypt(&self, ciphertext: &[u8]) -> DomainResult<String> {
            Ok(String::from_utf8(ciphertext.to_vec()).unwrap())
        }
    }

    struct StubFiring {
        job: Job,
        fired: Mutex<u32>,
    }

    #[async_trait]
    impl TriggerFiring for StubFiring {
        async fn fire(
            &self,
            _: &TriggerId,
            _: Option<&serde_json::Value>,
            _: Option<&str>,
        ) -> DomainResult<Job> {
            *self.fired.lock().unwrap() += 1;
            Ok(self.job.clone())
        }
    }

    struct Harness {
        ingress: WebhookIngressUseCases,
        deliveries: Arc<StubDeliveries>,
        firing: Arc<StubFiring>,
        trigger_id: TriggerId,
    }

    fn harness() -> Harness {
        use crate::test_support::{
            jobs::job, organizations::org, pipelines::pipeline, projects::project,
        };
        let trigger = Trigger::create(
            PipelineId::new("p"),
            TriggerName::new("hook").unwrap(),
            TriggerSource::Webhook(WebhookSpec::new(None).unwrap()),
            vec![],
        )
        .unwrap();
        let trigger_id = trigger.id().clone();
        let deliveries = Arc::new(StubDeliveries {
            recorded: Mutex::new(vec![]),
        });
        let firing = Arc::new(StubFiring {
            job: job(&pipeline(&project(&org("o"), "p"))),
            fired: Mutex::new(0),
        });
        let ingress = WebhookIngressUseCases::new(
            Arc::new(StubRepo { trigger }),
            deliveries.clone(),
            Arc::new(PlainCipher),
            firing.clone(),
        );
        Harness {
            ingress,
            deliveries,
            firing,
            trigger_id,
        }
    }

    fn signed(signature: &'static str) -> impl for<'a> Fn(&'a str) -> Option<String> + Sync {
        move |name: &str| (name == DEFAULT_SIGNATURE_HEADER).then(|| signature.to_string())
    }

    #[tokio::test]
    async fn signed_ping_answers_without_firing_or_recording() {
        let h = harness();
        let outcome = h
            .ingress
            .ingest(&h.trigger_id, &signed(SIG), Some("d-1"), Some("ping"), MSG)
            .await
            .unwrap();
        assert!(matches!(outcome, IngestOutcome::Ping));
        assert_eq!(*h.firing.fired.lock().unwrap(), 0);
        assert!(h.deliveries.recorded.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn ping_event_name_is_trimmed_and_case_insensitive() {
        let h = harness();
        let outcome = h
            .ingress
            .ingest(&h.trigger_id, &signed(SIG), None, Some(" Ping "), MSG)
            .await
            .unwrap();
        assert!(matches!(outcome, IngestOutcome::Ping));
        assert_eq!(*h.firing.fired.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn mis_signed_ping_is_rejected_before_the_event_check() {
        let h = harness();
        let err = h
            .ingress
            .ingest(&h.trigger_id, &signed("sha256=00"), None, Some("ping"), MSG)
            .await
            .unwrap_err();
        assert!(matches!(err, WebhookError::BadSignature));
        assert_eq!(*h.firing.fired.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn unsigned_ping_is_rejected_before_the_event_check() {
        let h = harness();
        let err = h
            .ingress
            .ingest(&h.trigger_id, &|_: &str| None, None, Some("ping"), MSG)
            .await
            .unwrap_err();
        assert!(matches!(err, WebhookError::BadSignature));
        assert_eq!(*h.firing.fired.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn other_events_and_no_event_still_fire() {
        let h = harness();
        let push = h
            .ingress
            .ingest(&h.trigger_id, &signed(SIG), Some("d-1"), Some("push"), MSG)
            .await
            .unwrap();
        assert!(matches!(push, IngestOutcome::Fired(_)));
        let bare = h
            .ingress
            .ingest(&h.trigger_id, &signed(SIG), Some("d-2"), None, MSG)
            .await
            .unwrap();
        assert!(matches!(bare, IngestOutcome::Fired(_)));
        assert_eq!(*h.firing.fired.lock().unwrap(), 2);
    }

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
