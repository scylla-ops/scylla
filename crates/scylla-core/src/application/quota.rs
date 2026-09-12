//! The quota extension point, as seen from the use cases.
//!
//! [`QuotaPolicy`] is declared in `scylla-extension` and implemented by the
//! edition binary; a use case that creates a metered resource holds it as a
//! trait object and asks before creating. This module holds the two things the
//! core adds around the trait: the Community default, and the translation of a
//! refusal into the domain error the surfaces already map.

use crate::domain::errors::{DomainError, DomainResult};
use async_trait::async_trait;
use scylla_extension::{QuotaDecision, QuotaError, QuotaPolicy, QuotaUsage, Resource};

/// The Community Edition policy: nothing is metered, nothing is consulted.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnlimitedQuota;

#[async_trait]
impl QuotaPolicy for UnlimitedQuota {
    async fn check(&self, _resource: Resource, _scope: &str) -> Result<QuotaDecision, QuotaError> {
        Ok(QuotaDecision::Allow)
    }

    async fn usage(
        &self,
        _resource: Resource,
        _scope: &str,
    ) -> Result<Option<QuotaUsage>, QuotaError> {
        Ok(None)
    }
}

/// Turn a policy's answer into the use case's `Result`.
///
/// `Allow` passes. `Deny` becomes [`DomainError::QuotaExceeded`] (mapped to gRPC
/// `RESOURCE_EXHAUSTED` by the handlers), carrying the message the frontend
/// shows verbatim, with the policy's upgrade hint appended when it gives one. A
/// policy that could not answer is an infrastructure failure of the operation,
/// not a decision.
pub fn enforce(decision: Result<QuotaDecision, QuotaError>) -> DomainResult<()> {
    match decision {
        Ok(QuotaDecision::Allow) => Ok(()),
        Ok(QuotaDecision::Deny {
            resource,
            limit,
            current,
            upgrade_hint,
        }) => {
            let mut message =
                format!("{resource} quota reached for this organization ({current}/{limit})");
            if let Some(hint) = upgrade_hint {
                message.push_str(". ");
                message.push_str(&hint);
            }
            Err(DomainError::quota_exceeded(message))
        }
        Err(e) => Err(DomainError::infrastructure(format!(
            "quota policy failed: {e}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unlimited_allows_and_reports_nothing_metered() {
        let policy = UnlimitedQuota;
        assert_eq!(
            policy.check(Resource::Project, "org").await,
            Ok(QuotaDecision::Allow)
        );
        assert_eq!(policy.usage(Resource::Project, "org").await, Ok(None));
    }

    #[test]
    fn allow_passes() {
        assert!(enforce(Ok(QuotaDecision::Allow)).is_ok());
    }

    #[test]
    fn deny_is_quota_exceeded_with_the_user_facing_message() {
        let err = enforce(Ok(QuotaDecision::Deny {
            resource: Resource::Project,
            limit: 2,
            current: 2,
            upgrade_hint: None,
        }))
        .unwrap_err();
        assert!(matches!(
            &err,
            DomainError::QuotaExceeded(m) if m == "project quota reached for this organization (2/2)"
        ));
    }

    #[test]
    fn deny_appends_the_upgrade_hint() {
        let err = enforce(Ok(QuotaDecision::Deny {
            resource: Resource::Project,
            limit: 1,
            current: 1,
            upgrade_hint: Some("contact sales".into()),
        }))
        .unwrap_err();
        assert!(matches!(
            &err,
            DomainError::QuotaExceeded(m)
                if m == "project quota reached for this organization (1/1). contact sales"
        ));
    }

    #[test]
    fn a_failing_policy_is_an_infrastructure_error() {
        let err = enforce(Err(QuotaError("metering db down".into()))).unwrap_err();
        assert!(matches!(err, DomainError::Infrastructure(_)));
    }
}
