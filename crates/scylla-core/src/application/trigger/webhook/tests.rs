//! The webhook ingress through the engine, on stub ports.

use super::*;
use crate::application::SecretCipher;
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{PipelineId, TriggerId};
use crate::domain::job::Job;
use crate::domain::trigger::{Trigger, TriggerName, TriggerSource, WebhookSpec};
use crate::test_support::authz::{DenyingPermissionService, actions};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Mutex;

const KEY: &str = "key";
const MSG: &[u8] = b"The quick brown fox jumps over the lazy dog";
const SIG: &str = "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8";

struct StubRepo {
    trigger: Trigger,
}

#[async_trait]
impl TriggerRepository for StubRepo {
    async fn find_by_id(&self, id: &TriggerId) -> DomainResult<Trigger> {
        if id == self.trigger.id() {
            Ok(self.trigger.clone())
        } else {
            Err(DomainError::not_found("Trigger", id.to_string()))
        }
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

impl Harness {
    async fn ingest(
        &self,
        signature: Option<&str>,
        delivery_id: Option<&str>,
        event: Option<&str>,
    ) -> DomainResult<IngestOutcome> {
        self.ingest_to(&self.trigger_id, signature, delivery_id, event)
            .await
    }

    async fn ingest_to(
        &self,
        trigger_id: &TriggerId,
        signature: Option<&str>,
        delivery_id: Option<&str>,
        event: Option<&str>,
    ) -> DomainResult<IngestOutcome> {
        let headers = signature
            .map(|s| {
                let name = DEFAULT_SIGNATURE_HEADER.to_ascii_lowercase();
                HashMap::from([(name, s.to_string())])
            })
            .unwrap_or_default();
        let command = IngestWebhook {
            trigger_id: trigger_id.clone(),
            headers,
            delivery_id: delivery_id.map(str::to_string),
            event: event.map(str::to_string),
            body: MSG.to_vec(),
        };
        actions(Arc::new(DenyingPermissionService::new()))
            .run(&self.ingress, &CallerContext::Anonymous, command)
            .await
    }
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

#[tokio::test]
async fn signed_ping_answers_without_firing_or_recording() {
    let h = harness();
    let outcome = h
        .ingest(Some(SIG), Some("d-1"), Some("ping"))
        .await
        .unwrap();
    assert!(matches!(outcome, IngestOutcome::Ping));
    assert_eq!(*h.firing.fired.lock().unwrap(), 0);
    assert!(h.deliveries.recorded.lock().unwrap().is_empty());
}

#[tokio::test]
async fn ping_event_name_is_trimmed_and_case_insensitive() {
    let h = harness();
    let outcome = h.ingest(Some(SIG), None, Some(" Ping ")).await.unwrap();
    assert!(matches!(outcome, IngestOutcome::Ping));
    assert_eq!(*h.firing.fired.lock().unwrap(), 0);
}

#[tokio::test]
async fn mis_signed_ping_is_rejected_before_the_event_check() {
    let h = harness();
    let err = h
        .ingest(Some("sha256=00"), None, Some("ping"))
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::Unauthorized(_)));
    assert_eq!(*h.firing.fired.lock().unwrap(), 0);
}

#[tokio::test]
async fn unsigned_ping_is_rejected_before_the_event_check() {
    let h = harness();
    let err = h.ingest(None, None, Some("ping")).await.unwrap_err();
    assert!(matches!(err, DomainError::Unauthorized(_)));
    assert_eq!(*h.firing.fired.lock().unwrap(), 0);
}

#[tokio::test]
async fn other_events_and_no_event_still_fire() {
    let h = harness();
    let push = h
        .ingest(Some(SIG), Some("d-1"), Some("push"))
        .await
        .unwrap();
    assert!(matches!(push, IngestOutcome::Fired(_)));
    let bare = h.ingest(Some(SIG), Some("d-2"), None).await.unwrap();
    assert!(matches!(bare, IngestOutcome::Fired(_)));
    assert_eq!(*h.firing.fired.lock().unwrap(), 2);
}

#[tokio::test]
async fn a_repeated_delivery_is_a_duplicate_and_fires_once() {
    let h = harness();
    h.ingest(Some(SIG), Some("d-1"), None).await.unwrap();
    let again = h.ingest(Some(SIG), Some(" d-1 "), None).await.unwrap();
    assert!(matches!(again, IngestOutcome::Duplicate));
    assert_eq!(*h.firing.fired.lock().unwrap(), 1);
}

#[tokio::test]
async fn an_unknown_trigger_is_not_found_without_asking_a_permission() {
    let h = harness();
    let err = h
        .ingest_to(&TriggerId::new("missing"), Some(SIG), None, None)
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::NotFound { .. }));
    assert!(h.deliveries.recorded.lock().unwrap().is_empty());
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
