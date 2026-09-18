//! The permission check is a stage, not a hook: it runs with zero extensions registered, and its
//! output type is the proof that it ran.

use crate::action::{Authorized, Describe, Requested};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::domain::permission::Permission;
use crate::stage::{Authorize, Run};
use async_trait::async_trait;
use std::sync::Arc;

/// Only `AuthorizeStage` builds one, and `Requested::authorized` consumes it, so an
/// `Authorized<C>` can only come from the check.
#[derive(Debug, Clone, Copy)]
pub struct Granted(());

/// `Err(Forbidden)` and never `Ok(false)`: the `PermissionService` contract, restated here so the
/// pipeline does not depend on the access model crate.
#[async_trait]
pub trait Authorizer: Send + Sync {
    async fn authorize(&self, caller: &CallerContext, permission: Permission) -> DomainResult<()>;
}

pub struct AuthorizeStage {
    authorizer: Arc<dyn Authorizer>,
}

impl AuthorizeStage {
    #[must_use]
    pub fn new(authorizer: Arc<dyn Authorizer>) -> Self {
        Self { authorizer }
    }
}

#[async_trait]
impl<C: Describe> Run<Authorize<C>> for AuthorizeStage {
    async fn run(&self, input: Requested<C>) -> DomainResult<Authorized<C>> {
        self.authorizer
            .authorize(input.caller(), input.permission().clone())
            .await?;
        Ok(input.authorized(Granted(())))
    }
}
