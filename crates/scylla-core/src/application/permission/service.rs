use crate::application::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::policy::Policy;

/// Identity-aware permission check. The `caller` shape is Cedar-compatible
/// (User / Service / Anonymous). Single-method trait — Cedar policies are
/// static text compiled into the binary, so policy CRUD belongs to ops/code
/// review, not to a runtime API.
#[async_trait::async_trait]
pub trait PermissionService: Send + Sync {
    async fn check(&self, caller: &CallerContext, policy: Policy) -> DomainResult<bool>;
}
