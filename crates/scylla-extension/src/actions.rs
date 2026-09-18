//! The one engine. A use case implements `Run<Prepare<C>>` and `Run<Persist<C>>` for its
//! commands, `Run<Fetch<Q>>` for its queries, and an adapter calls `send` or `query` with it.
//! The use case has no hook code, no permission code and no method of its own.

use crate::action::{Command, Describe, Query, Requested};
use crate::authz::{AuthorizeStage, Authorizer};
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use crate::hooks::Hooks;
use crate::stage::{Authorize, Fetch, Persist, Prepare, Run};
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

    /// Authorize, prepare, persist, each through the hooks. The runner is one object for the two
    /// use-case stages; the turbofish on `run` selects which impl a stage uses.
    pub async fn send<C: Command, R>(
        &self,
        runner: &R,
        caller: &CallerContext,
        command: C,
    ) -> DomainResult<C::Committed>
    where
        R: Run<Prepare<C>> + Run<Persist<C>>,
    {
        let requested = Requested::new(caller.clone(), command);
        let span = Self::span("command", &requested);
        async {
            let authorized = self
                .hooks
                .run::<Authorize<C>>(&self.authorize, requested)
                .await?;
            let prepared = self.hooks.run::<Prepare<C>>(runner, authorized).await?;
            let committed = self.hooks.run::<Persist<C>>(runner, prepared).await?;
            Ok(committed.into_outcome())
        }
        .instrument(span)
        .await
    }

    /// Authorize, fetch, each through the hooks.
    pub async fn query<Q: Query, R>(
        &self,
        runner: &R,
        caller: &CallerContext,
        query: Q,
    ) -> DomainResult<Q::Output>
    where
        R: Run<Fetch<Q>>,
    {
        let requested = Requested::new(caller.clone(), query);
        let span = Self::span("query", &requested);
        async {
            let authorized = self
                .hooks
                .run::<Authorize<Q>>(&self.authorize, requested)
                .await?;
            let fetched = self.hooks.run::<Fetch<Q>>(runner, authorized).await?;
            Ok(fetched.into_output())
        }
        .instrument(span)
        .await
    }

    // One span shape for every action: the id groups the stages, the permission names the
    // action and its resource. Use cases carry no `#[instrument]` of their own.
    fn span<C: Describe>(kind: &'static str, requested: &Requested<C>) -> tracing::Span {
        info_span!(
            "action",
            kind,
            action = %requested.id(),
            caller = %requested.caller(),
            permission = requested.permission().key(),
            resource = %requested.permission().resource(),
        )
    }
}
