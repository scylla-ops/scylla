use super::command::Command;
use super::envelope::Envelope;
use crate::authz::Granted;
use crate::domain::caller::CallerContext;
use crate::domain::errors::DomainResult;
use std::fmt;
use std::sync::Arc;

pub struct Requested<C> {
    pub(super) env: Arc<Envelope<C>>,
}

pub struct Authorized<C> {
    pub(super) env: Arc<Envelope<C>>,
}

pub struct Prepared<C: Command> {
    pub(super) env: Arc<Envelope<C>>,
    staged: C::Staged,
}

pub struct Committed<C: Command> {
    pub(super) env: Arc<Envelope<C>>,
    outcome: C::Committed,
}

impl<C: Command> Requested<C> {
    /// Only `Actions::send` mints an envelope: an adapter cannot enter the pipeline halfway.
    pub(crate) fn new(caller: CallerContext, command: C) -> Self {
        Self {
            env: Arc::new(Envelope::new(caller, command)),
        }
    }

    #[must_use]
    pub fn authorized(self, _: Granted) -> Authorized<C> {
        Authorized { env: self.env }
    }
}

impl<C: Command> Authorized<C> {
    pub fn prepared(self, staged: C::Staged) -> Prepared<C> {
        Prepared {
            env: self.env,
            staged,
        }
    }
}

impl<C: Command> Prepared<C> {
    pub fn staged(&self) -> &C::Staged {
        &self.staged
    }

    /// The only way to `Committed`. The closure receives the staged value by move and must return
    /// the committed one; the store writes inside it. A closure that does not write is a
    /// simulation, which is what a dry-run `Wrap` does.
    pub async fn commit<F>(self, write: F) -> DomainResult<Committed<C>>
    where
        F: AsyncFnOnce(C::Staged) -> DomainResult<C::Committed>,
    {
        let outcome = write(self.staged).await?;
        Ok(Committed {
            env: self.env,
            outcome,
        })
    }
}

impl<C: Command> Committed<C> {
    pub fn outcome(&self) -> &C::Committed {
        &self.outcome
    }

    pub fn into_outcome(self) -> C::Committed {
        self.outcome
    }
}

impl<C: Command> fmt::Debug for Committed<C>
where
    C::Committed: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Committed")
            .field("id", self.env.id())
            .field("caller", self.env.caller())
            .field("permission", self.env.permission())
            .field("outcome", &self.outcome)
            .finish()
    }
}
