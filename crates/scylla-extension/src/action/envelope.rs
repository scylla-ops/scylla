use super::command::Command;
use super::id::ActionId;
use crate::domain::caller::CallerContext;
use crate::domain::clock;
use crate::domain::permission::Permission;
use chrono::{DateTime, Utc};

/// Built once per `send` and shared by every phase. The permission is computed here, once, so a
/// hook reads it without calling the command again.
pub struct Envelope<C> {
    id: ActionId,
    at: DateTime<Utc>,
    caller: CallerContext,
    permission: Permission,
    command: C,
}

impl<C: Command> Envelope<C> {
    pub(super) fn new(caller: CallerContext, command: C) -> Self {
        Self {
            id: ActionId::generate(),
            at: clock::now(),
            permission: command.permission(),
            caller,
            command,
        }
    }
}

impl<C> Envelope<C> {
    pub fn id(&self) -> &ActionId {
        &self.id
    }

    pub fn at(&self) -> DateTime<Utc> {
        self.at
    }

    pub fn caller(&self) -> &CallerContext {
        &self.caller
    }

    pub fn permission(&self) -> &Permission {
        &self.permission
    }

    pub fn command(&self) -> &C {
        &self.command
    }
}
