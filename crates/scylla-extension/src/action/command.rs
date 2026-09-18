use crate::domain::permission::Permission;

/// What every action has: the permission the authorize stage checks.
pub trait Describe: Send + Sync + 'static {
    fn permission(&self) -> Permission;
}

/// A write. The two payload types say whether the thing is in the store: `Draft<T>` is not,
/// `T` is, `Deleted<T>` was.
pub trait Command: Describe {
    type Staged: Send + Sync + 'static;
    type Committed: Send + Sync + 'static;
}

/// A read. It has no staged value and no write, so it takes two stages instead of three.
pub trait Query: Describe {
    type Output: Send + Sync + 'static;
}
