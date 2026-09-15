use crate::domain::ids::{AppId, UserId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerContext {
    User(UserId),
    App(AppId),
    Service(ServiceIdentity),
    Anonymous,
}

/// Sealed: only the factories below build one, so a handler cannot forge a service caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceIdentity {
    name: &'static str,
}

impl ServiceIdentity {
    #[must_use]
    pub fn recorder() -> Self {
        Self { name: "recorder" }
    }

    #[must_use]
    pub fn bootstrap() -> Self {
        Self { name: "bootstrap" }
    }

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        self.name
    }
}

impl CallerContext {
    #[must_use]
    pub fn to_entity_uid(&self) -> String {
        match self {
            Self::User(id) => format!("Scylla::User::\"{}\"", id.as_str()),
            Self::App(id) => format!("Scylla::App::\"{}\"", id.as_str()),
            Self::Service(svc) => format!("Scylla::Service::\"{}\"", svc.as_str()),
            Self::Anonymous => "Scylla::Anonymous::\"*\"".to_string(),
        }
    }
}

impl std::fmt::Display for CallerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(id) => write!(f, "user:{}", id.as_str()),
            Self::App(id) => write!(f, "app:{}", id.as_str()),
            Self::Service(svc) => write!(f, "service:{}", svc.as_str()),
            Self::Anonymous => write!(f, "anonymous"),
        }
    }
}
