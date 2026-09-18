use crate::domain::permission::Permission;
use crate::path::{Kind, Read, Write};

/// What every action has: the permission the authorize stage checks, and the path it takes
/// after it. The path is declared, not inferred: coherence forbids "every `Command`" and "every
/// `Query`" as two impls of one trait, since nothing stops a type from being both.
pub trait Describe: Send + Sync + 'static {
    type Path: Kind;

    fn permission(&self) -> Permission;
}

/// A write, on the `Write` path. The two payload types say whether the thing is in the store:
/// `Draft<T>` is not, `T` is, `Deleted<T>` was.
pub trait Command: Describe<Path = Write> {
    type Staged: Send + Sync + 'static;
    type Committed: Send + Sync + 'static;
}

/// A read, on the `Read` path. It has no staged value and no write.
pub trait Query: Describe<Path = Read> {
    type Output: Send + Sync + 'static;
}
