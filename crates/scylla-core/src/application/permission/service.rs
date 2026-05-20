use crate::application::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::Permission;

/// Identity-aware authorization check. Given the caller and the operation they
/// want to perform (`Permission` carries both the action and the concrete
/// resource), decide allow/deny. The production adapter is Cedar-backed; the
/// trait stays free of any Cedar type so the domain/application layers don't
/// depend on the engine.
#[async_trait::async_trait]
pub trait PermissionService: Send + Sync {
    async fn check(&self, caller: &CallerContext, perm: Permission) -> DomainResult<bool>;
}
