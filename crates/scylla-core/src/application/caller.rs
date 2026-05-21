use crate::domain::entities::{AppId, UserId};

/// Identity of the principal invoking a use-case.
///
/// Shape mirrors a Cedar `principal`: `User` → `Scylla::User::"<id>"`,
/// `App` → `Scylla::App::"<id>"`, `Service` → `Scylla::Service::"<name>"`,
/// `Anonymous` → no entity. The `Service` variant is **sealed** —
/// `ServiceIdentity` can only be built via the named factory functions
/// (`recorder()`, `bootstrap()`), preventing a downstream module from forging
/// an arbitrary service caller mid-chain. An `App` is a machine principal
/// (agent / automation) authenticated by an app token; it carries scoped grants
/// just like a user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerContext {
    User(UserId),
    App(AppId),
    Service(ServiceIdentity),
    Anonymous,
}

/// Sealed service principal. Constructible only through the named factories
/// below — handlers and use-cases cannot synthesise an arbitrary service
/// identity from the inside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceIdentity {
    name: &'static str,
}

impl ServiceIdentity {
    /// Used by the in-process recorder listeners when persisting broker events.
    #[must_use]
    pub fn recorder() -> Self {
        Self { name: "recorder" }
    }

    /// Used by the control-plane bootstrap path that creates the admin user and
    /// assigns the `admin` role on first boot. Runs exactly once per fresh DB.
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
    /// Cedar-style entity UID (e.g. `Scylla::User::"01h…"`). Used by
    /// `CedarPermissionService` to build the request principal.
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

/// Compact, human-readable label for audit logs (e.g. `user:01h…`,
/// `service:recorder`, `anonymous`).
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
