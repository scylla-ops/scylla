//! The one engine. A use case implements `Run<Prepare<C>>` and `Run<Persist<C>>` for its
//! commands, `Run<Fetch<Q>>` for its queries, and an adapter calls `run` with it for both. The
//! use case has no hook code, no permission code and no method of its own.

use crate::action::{Describe, Requested};
use crate::authz::{AuthorizeStage, Authorizer};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::hooks::Hooks;
use crate::path::{Kind, Path};
use crate::stage::Authorize;
use std::sync::Arc;
use tracing::{Instrument, info_span};

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

    /// Authorize, then the action's path, each stage through the hooks. The runner is one object
    /// for every stage of the action; the turbofish on `Hooks::run` selects which impl a stage
    /// uses. Every action runs in one span; use cases carry no `#[instrument]` of their own.
    pub async fn run<A: Describe, R>(
        &self,
        runner: &R,
        caller: &CallerContext,
        action: A,
    ) -> DomainResult<<A::Path as Path<A, R>>::Output>
    where
        A::Path: Path<A, R>,
    {
        let requested = Requested::new(caller.clone(), action);
        let span = info_span!(
            "action",
            kind = <A::Path as Kind>::NAME,
            action = %requested.id(),
            caller = %requested.caller(),
            permission = requested.permission().key(),
            resource = %requested.permission().resource(),
        );
        async {
            let authorized = self
                .hooks
                .run::<Authorize<A>>(&self.authorize, requested)
                .await?;
            A::Path::run(&self.hooks, runner, authorized).await
        }
        .instrument(span)
        .await
    }
}
