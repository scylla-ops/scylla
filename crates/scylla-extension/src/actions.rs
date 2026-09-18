//! The one engine. A use case implements `Run<Prepare<C>>` and `Run<Persist<C>>` for its
//! commands and calls `send`; it has no hook code and no permission code.

use crate::action::{Command, Committed, Requested};
use crate::authz::{AuthorizeStage, Authorizer};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::hooks::Hooks;
use crate::stage::{Authorize, Persist, Prepare, Run};
use std::sync::Arc;

pub struct Actions {
    authorize: AuthorizeStage,
    hooks: Arc<Hooks>,
}

impl Actions {
    #[must_use]
    pub fn new(authorizer: Arc<dyn Authorizer>, hooks: Arc<Hooks>) -> Self {
        Self {
            authorize: AuthorizeStage::new(authorizer),
            hooks,
        }
    }

    /// Authorize, prepare, persist, each through the hooks. The runner is the same object for the
    /// two use-case stages; the turbofish on `run` selects which impl a stage uses.
    pub async fn send<C: Command, R>(
        &self,
        runner: &R,
        caller: &CallerContext,
        command: C,
    ) -> DomainResult<Committed<C>>
    where
        R: Run<Prepare<C>> + Run<Persist<C>>,
    {
        let requested = Requested::new(caller.clone(), command);
        let authorized = self
            .hooks
            .run::<Authorize<C>>(&self.authorize, requested)
            .await?;
        let prepared = self.hooks.run::<Prepare<C>>(runner, authorized).await?;
        self.hooks.run::<Persist<C>>(runner, prepared).await
    }
}
