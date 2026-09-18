//! The event: one command, one envelope, four phases that only transitions can produce.

mod command;
mod envelope;
mod erased;
mod id;
mod phase;
mod value;

pub use command::Command;
pub use envelope::Envelope;
pub use erased::{Action, Phase};
pub use id::ActionId;
pub use phase::{Authorized, Committed, Prepared, Requested};
pub use value::{Deleted, Draft};
