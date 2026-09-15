use crate::domain::errors::{DomainError, DomainResult};
use async_trait::async_trait;
use scylla_extension::{Extensions, QuotaDecision, QuotaError, QuotaPolicy, QuotaUsage, Resource};
use std::sync::Arc;

#[must_use]
pub fn quota_policy(extensions: &Extensions) -> Arc<dyn QuotaPolicy> {
    extensions
        .get::<dyn QuotaPolicy>()
        .unwrap_or_else(|| Arc::new(UnlimitedQuota))
}

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
    async fn the_community_default_is_unlimited() {
        let policy = quota_policy(&Extensions::new());
        assert_eq!(
            policy.check(Resource::Project, "org").await,
            Ok(QuotaDecision::Allow)
        );
        assert_eq!(policy.usage(Resource::Project, "org").await, Ok(None));
    }

    #[test]
    fn a_registered_policy_wins_over_the_default() {
        let extensions = Extensions::new().with::<dyn QuotaPolicy>(Arc::new(UnlimitedQuota));
        let registered = extensions.get::<dyn QuotaPolicy>().unwrap();
        assert!(Arc::ptr_eq(&quota_policy(&extensions), &registered));
    }

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
