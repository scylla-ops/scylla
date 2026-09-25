pub mod commands;

pub use commands::{Delivery, IngestWebhook};

use crate::application::SecretCipher;
use crate::application::trigger::delivery::TriggerDeliveryRepository;
use crate::application::trigger::fire::TriggerFiring;
use crate::application::trigger::repository::TriggerRepository;
use crate::domain::ids::JobId;
use derive_more::Constructor;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::sync::Arc;
use subtle::ConstantTimeEq;

pub const DEFAULT_SIGNATURE_HEADER: &str = "X-Scylla-Signature-256";
pub const PING_EVENT: &str = "ping";

#[derive(Debug)]
pub enum IngestOutcome {
    Fired(JobId),
    Duplicate,
    Ping,
}

/// The webhook ingress's stage runners, one block per action in `commands.rs`. `IngestWebhook`
/// is `Public`: the signature is the credential, and it is verified before any write, so an
/// unauthenticated caller cannot pollute the dedupe table.
#[derive(Constructor)]
pub struct WebhookIngressUseCases {
    pub(super) trigger_repo: Arc<dyn TriggerRepository>,
    pub(super) delivery_repo: Arc<dyn TriggerDeliveryRepository>,
    pub(super) cipher: Arc<dyn SecretCipher>,
    pub(super) firing: Arc<dyn TriggerFiring>,
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
mod tests;
