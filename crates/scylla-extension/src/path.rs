//! Where one action's chain forks. `Authorize` is the same for every action; what follows is
//! decided by the type: a `Write` goes through prepare and persist, a `Read` through fetch.

use crate::action::{Authorized, Command, Describe, Query};
use crate::domain::errors::DomainResult;
use crate::hooks::Hooks;
use crate::stage::{Fetch, Persist, Prepare, Run};
use async_trait::async_trait;

mod sealed {
    pub trait Sealed {}
}

/// Sealed: a third path is a change to the pipeline, not to a use case.
pub trait Kind: sealed::Sealed + Send + Sync + 'static {
    const NAME: &'static str;
}

pub struct Write;
pub struct Read;

impl sealed::Sealed for Write {}
impl sealed::Sealed for Read {}

impl Kind for Write {
    const NAME: &'static str = "command";
}

impl Kind for Read {
    const NAME: &'static str = "query";
}

/// The rest of the chain for an action `A` run by `R`: the stages after `Authorize`, the bound
/// the runner must satisfy, and what comes out. One impl per path.
#[async_trait]
pub trait Path<A: Describe, R>: Kind {
    type Output;

    async fn run(
        hooks: &Hooks,
        runner: &R,
        authorized: Authorized<A>,
    ) -> DomainResult<Self::Output>;
}

#[async_trait]
impl<C, R> Path<C, R> for Write
where
    C: Command,
    R: Run<Prepare<C>> + Run<Persist<C>>,
{
    type Output = C::Committed;

    async fn run(
        hooks: &Hooks,
        runner: &R,
        authorized: Authorized<C>,
    ) -> DomainResult<C::Committed> {
        let prepared = hooks.run::<Prepare<C>>(runner, authorized).await?;
        let committed = hooks.run::<Persist<C>>(runner, prepared).await?;
        Ok(committed.into_outcome())
    }
}

#[async_trait]
impl<Q, R> Path<Q, R> for Read
where
    Q: Query,
    R: Run<Fetch<Q>>,
{
    type Output = Q::Output;

    async fn run(hooks: &Hooks, runner: &R, authorized: Authorized<Q>) -> DomainResult<Q::Output> {
        let fetched = hooks.run::<Fetch<Q>>(runner, authorized).await?;
        Ok(fetched.into_output())
    }
}
