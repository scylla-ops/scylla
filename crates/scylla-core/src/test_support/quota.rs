//! A `QuotaPolicy` double for use-case tests.
//!
//! [`DenyAfter`] allows a fixed number of checks per scope and denies from then
//! on, counting the checks it was asked rather than any stored resource. It is
//! enough to prove a use case asks the policy and honours a `Deny`.

use async_trait::async_trait;
use scylla_extension::{QuotaDecision, QuotaError, QuotaPolicy, QuotaUsage, Resource};
use std::collections::HashMap;
use std::sync::Mutex;

/// Allows the first `limit` checks for a scope, denies every later one.
pub struct DenyAfter {
    limit: u64,
    seen: Mutex<HashMap<String, u64>>,
}

impl DenyAfter {
    #[must_use]
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            seen: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl QuotaPolicy for DenyAfter {
    async fn check(&self, resource: Resource, scope: &str) -> Result<QuotaDecision, QuotaError> {
        let mut seen = self.seen.lock().unwrap();
        let current = seen.entry(scope.to_owned()).or_insert(0);
        if *current >= self.limit {
            return Ok(QuotaDecision::Deny {
                resource,
                limit: self.limit,
                current: *current,
                upgrade_hint: None,
            });
        }
        *current += 1;
        Ok(QuotaDecision::Allow)
    }

    async fn usage(
        &self,
        resource: Resource,
        scope: &str,
    ) -> Result<Option<QuotaUsage>, QuotaError> {
        let current = self.seen.lock().unwrap().get(scope).copied().unwrap_or(0);
        Ok(Some(QuotaUsage {
            resource,
            current,
            limit: Some(self.limit),
        }))
    }
}
