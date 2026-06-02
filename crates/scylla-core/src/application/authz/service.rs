use crate::application::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::action::Action;

/// Identity-aware authorization check. Given the caller and the operation they
/// want to perform (`Action` carries both the action and the concrete
/// resource), allow or deny. The production adapter is Cedar-backed; the trait
/// stays free of any Cedar type so the domain/application layers don't depend on
/// the engine.
///
/// Returns `Ok(())` when permitted and `Err(DomainError::Forbidden)` when
/// denied — there is no `Ok(false)`. A `Result<()>` (rather than `bool`) makes
/// the API fail-closed by construction: a caller can only proceed by handling
/// the error, so it's impossible to accidentally treat a denial as success (the
/// classic `if check(...) {}` / `unwrap_or(true)` foot-gun).
#[async_trait::async_trait]
pub trait PermissionService: Send + Sync {
    async fn check(&self, caller: &CallerContext, perm: Action) -> DomainResult<()>;
}
