use super::command::Command;
use super::envelope::Envelope;
use super::id::ActionId;
use super::phase::{Authorized, Committed, Prepared, Requested};
use crate::domain::caller::CallerContext;
use crate::domain::permission::Permission;
use chrono::{DateTime, Utc};
use std::any::Any;
use std::ops::Deref;
use std::sync::Arc;

/// The erased view of any phase. An erased hook (`Policy`, `Around`, `Observer`) fires for
/// commands that do not exist yet, so it only gets what every command has; `downcast_ref` reaches
/// the typed phase when the hook knows the command.
pub trait Action: Send + Sync {
    fn id(&self) -> &ActionId;
    fn at(&self) -> DateTime<Utc>;
    fn caller(&self) -> &CallerContext;
    fn permission(&self) -> &Permission;
    fn as_any(&self) -> &dyn Any;
}

/// A phase can hand out its envelope, which is what an observer sees when a stage failed after
/// consuming the input.
pub trait Phase: Action {
    fn envelope(&self) -> Arc<dyn Action>;
}

impl dyn Action + '_ {
    #[must_use]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }
}

impl<C: Command> Action for Envelope<C> {
    fn id(&self) -> &ActionId {
        Envelope::id(self)
    }

    fn at(&self) -> DateTime<Utc> {
        Envelope::at(self)
    }

    fn caller(&self) -> &CallerContext {
        Envelope::caller(self)
    }

    fn permission(&self) -> &Permission {
        Envelope::permission(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

macro_rules! phase {
    ($($phase:ident),*) => {
        $(
            impl<C: Command> Deref for $phase<C> {
                type Target = Envelope<C>;

                fn deref(&self) -> &Envelope<C> {
                    &self.env
                }
            }

            impl<C: Command> Action for $phase<C> {
                fn id(&self) -> &ActionId {
                    self.env.id()
                }

                fn at(&self) -> DateTime<Utc> {
                    self.env.at()
                }

                fn caller(&self) -> &CallerContext {
                    self.env.caller()
                }

                fn permission(&self) -> &Permission {
                    self.env.permission()
                }

                fn as_any(&self) -> &dyn Any {
                    self
                }
            }

            impl<C: Command> Phase for $phase<C> {
                fn envelope(&self) -> Arc<dyn Action> {
                    self.env.clone()
                }
            }
        )*
    };
}

phase!(Requested, Authorized, Prepared, Committed);
