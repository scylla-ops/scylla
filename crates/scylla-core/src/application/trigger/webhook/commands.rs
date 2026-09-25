//! The webhook ingress's writes. One block per command, in the order it runs: the struct, its
//! access, its payload types, what `Prepare` builds, what `Persist` writes. An unknown, disabled
//! or non-webhook trigger is one opaque `NotFound`, and a missing or wrong signature is
//! `Unauthorized`; every other failure is `Internal`, so it never reads as one of the two.

use super::{
    DEFAULT_SIGNATURE_HEADER, IngestOutcome, PING_EVENT, WebhookIngressUseCases, verify_signature,
};
use crate::domain::clock;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::TriggerId;
use crate::domain::trigger::TriggerSource;
use async_trait::async_trait;
use scylla_extension::{
    Access, Authorized, Command, Committed, Describe, Draft, Persist, Prepare, Prepared, Run,
};
use std::collections::HashMap;

/// No `Debug`: the headers carry the signature. `headers` is keyed by the lowercase name.
pub struct IngestWebhook {
    pub trigger_id: TriggerId,
    pub headers: HashMap<String, String>,
    pub delivery_id: Option<String>,
    pub event: Option<String>,
    pub body: Vec<u8>,
}

impl IngestWebhook {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }
}

/// A ping is answered without a write; an event is recorded under its dedupe key, then fired.
pub enum Delivery {
    Ping,
    Event {
        trigger_id: TriggerId,
        key: String,
        delivery_id: Option<String>,
        payload: Option<serde_json::Value>,
    },
}

impl Describe for IngestWebhook {
    fn access(&self) -> Access {
        Access::Public
    }
}

impl Command for IngestWebhook {
    type Staged = Draft<Delivery>;
    type Committed = IngestOutcome;
}

fn not_found(id: &TriggerId) -> DomainError {
    DomainError::not_found("Trigger", id.to_string())
}

fn internal(e: &DomainError) -> DomainError {
    DomainError::internal(e.to_string())
}

#[async_trait]
impl Run<Prepare<IngestWebhook>> for WebhookIngressUseCases {
    async fn run(&self, input: Authorized<IngestWebhook>) -> DomainResult<Prepared<IngestWebhook>> {
        let cmd = input.command();
        let trigger = match self.trigger_repo.find_by_id(&cmd.trigger_id).await {
            Ok(t) => t,
            Err(e) if e.is_not_found() => return Err(not_found(&cmd.trigger_id)),
            Err(e) => return Err(internal(&e)),
        };
        if !trigger.is_enabled() {
            return Err(not_found(&cmd.trigger_id));
        }
        let TriggerSource::Webhook(spec) = trigger.source() else {
            return Err(not_found(&cmd.trigger_id));
        };
        let header_name = spec.signature_header().unwrap_or(DEFAULT_SIGNATURE_HEADER);

        let secret = match self.trigger_repo.webhook_secret(&cmd.trigger_id).await {
            Ok(Some(enc)) => self.cipher.decrypt(&enc).map_err(|e| internal(&e))?,
            Ok(None) => {
                return Err(DomainError::internal(
                    "webhook trigger has no signing secret",
                ));
            }
            Err(e) => return Err(internal(&e)),
        };

        let bad_signature = || DomainError::unauthorized("invalid signature");
        let signature = cmd.header(header_name).ok_or_else(bad_signature)?;
        if !verify_signature(&secret, &cmd.body, signature) {
            return Err(bad_signature());
        }
        // Verified before the ping check: a wrong secret fails on GitHub's creation-time ping, not on the first push.
        if cmd
            .event
            .as_deref()
            .is_some_and(|e| e.trim().eq_ignore_ascii_case(PING_EVENT))
        {
            return Ok(input.prepared(Draft::new(Delivery::Ping)));
        }

        let key = cmd
            .delivery_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(signature)
            .to_string();
        let delivery = Delivery::Event {
            trigger_id: cmd.trigger_id.clone(),
            key,
            delivery_id: cmd.delivery_id.clone(),
            payload: serde_json::from_slice(&cmd.body).ok(),
        };
        Ok(input.prepared(Draft::new(delivery)))
    }
}

#[async_trait]
impl Run<Persist<IngestWebhook>> for WebhookIngressUseCases {
    async fn run(&self, input: Prepared<IngestWebhook>) -> DomainResult<Committed<IngestWebhook>> {
        input
            .commit(async |draft| {
                let Delivery::Event {
                    trigger_id,
                    key,
                    delivery_id,
                    payload,
                } = draft.into_inner()
                else {
                    return Ok(IngestOutcome::Ping);
                };
                let is_new = self
                    .delivery_repo
                    .record_or_detect(&trigger_id, &key, clock::now())
                    .await
                    .map_err(|e| internal(&e))?;
                if !is_new {
                    return Ok(IngestOutcome::Duplicate);
                }
                let job = self
                    .firing
                    .fire(&trigger_id, payload.as_ref(), delivery_id.as_deref())
                    .await
                    .map_err(|e| internal(&e))?;
                Ok(IngestOutcome::Fired(job.id().clone()))
            })
            .await
    }
}
