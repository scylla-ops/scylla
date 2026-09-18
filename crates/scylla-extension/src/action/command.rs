use crate::domain::permission::Permission;

/// A write, described once: the permission the authorize stage checks, the value the use case
/// stages and the value the store commits. The two types say whether the thing is in the store:
/// `Draft<T>` is not, `T` is, `Deleted<T>` was.
pub trait Command: Send + Sync + 'static {
    type Staged: Send + Sync + 'static;
    type Committed: Send + Sync + 'static;

    fn permission(&self) -> Permission;
}
