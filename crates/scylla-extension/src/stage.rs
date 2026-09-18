//! Three stages move the event; each is a type with an `In` and an `Out` phase.

use crate::action::{Action, Authorized, Command, Committed, Phase, Prepared, Requested};
use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StageKind {
    Authorize,
    Prepare,
    Persist,
}

impl StageKind {
    pub const ALL: [Self; 3] = [Self::Authorize, Self::Prepare, Self::Persist];
}

impl fmt::Display for StageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Authorize => "authorize",
            Self::Prepare => "prepare",
            Self::Persist => "persist",
        })
    }
}

pub trait Stage: Send + Sync + 'static {
    type In: Phase + Send + 'static;
    type Out: Action + Send + 'static;

    const KIND: StageKind;
}

/// `Requested<C>` to `Authorized<C>`. `AuthorizeStage` runs it for every command.
pub struct Authorize<C>(PhantomData<fn() -> C>);
/// `Authorized<C>` to `Prepared<C>`. The use case runs it: it reads and builds, it does not write.
pub struct Prepare<C>(PhantomData<fn() -> C>);
/// `Prepared<C>` to `Committed<C>`. The store runs it: it writes and nothing else.
pub struct Persist<C>(PhantomData<fn() -> C>);

impl<C: Command> Stage for Authorize<C> {
    type In = Requested<C>;
    type Out = Authorized<C>;

    const KIND: StageKind = StageKind::Authorize;
}

impl<C: Command> Stage for Prepare<C> {
    type In = Authorized<C>;
    type Out = Prepared<C>;

    const KIND: StageKind = StageKind::Prepare;
}

impl<C: Command> Stage for Persist<C> {
    type In = Prepared<C>;
    type Out = Committed<C>;

    const KIND: StageKind = StageKind::Persist;
}

/// What a stage implementation satisfies. The signature is fixed by `S`: the compiler, not the
/// author, decides the input and output phase.
#[async_trait]
pub trait Run<S: Stage>: Send + Sync {
    async fn run(&self, input: S::In) -> DomainResult<S::Out>;
}
