//! Conversions between the wire DTOs from `scylla.common.v1` / `scylla.authz.v1`
//! (id wrappers, `google.protobuf.Timestamp`, closed unions) and the plain
//! `String` ids / `chrono` timestamps / sum types the domain uses. Centralised
//! here so each mapper and handler site stays a one-liner.

use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;
use scylla_core::application::{Principal, Scope, ScopeKind};
use scylla_core::domain::entities::{AppId, OrganizationId, ProjectId, UserId};
use scylla_protocol::authz::v1::{
    Permission, PrincipalRef, ScopeKind as ProtoScopeKind, ScopeRef, principal_ref, scope_ref,
};
use scylla_protocol::common::v1 as common;
use scylla_protocol::common::v1::LogStream;
use tonic::Status;

/// A proto wrapper message (`common.v1.*Id` / `common.v1.Email`) — a single `value`.
pub trait Wrapper: Sized {
    fn wrap(value: String) -> Self;
    fn into_value(self) -> String;
}

macro_rules! impl_wrapper {
    ($($t:ty),+ $(,)?) => {$(
        impl Wrapper for $t {
            fn wrap(value: String) -> Self { Self { value } }
            fn into_value(self) -> String { self.value }
        }
    )+};
}

impl_wrapper!(
    common::UserId,
    common::OrganizationId,
    common::ProjectId,
    common::PipelineId,
    common::JobId,
    common::JobLogId,
    common::AppId,
    common::AppSecretId,
    common::InvitationId,
    common::NodeId,
    common::SecretId,
    common::TriggerId,
    common::GrantId,
    common::RoleId,
    common::Email,
);

/// Wrap a domain id/string into its proto wrapper, as the `Some(..)` a proto
/// message field expects.
pub fn wrap<T: Wrapper>(value: impl Into<String>) -> Option<T> {
    Some(T::wrap(value.into()))
}

/// Extract the inner string from a required proto wrapper field, erroring with
/// `InvalidArgument` if the client omitted it.
pub fn required<T: Wrapper>(field: Option<T>, name: &str) -> Result<String, Status> {
    field
        .map(T::into_value)
        .ok_or_else(|| Status::invalid_argument(format!("missing {name}")))
}

/// Extract an optional proto wrapper field into `Option<String>`.
pub fn optional<T: Wrapper>(field: Option<T>) -> Option<String> {
    field.map(T::into_value)
}

/// Domain timestamp → proto `Timestamp`, as the `Some(..)` a field expects.
#[must_use]
pub fn ts(dt: DateTime<Utc>) -> Option<Timestamp> {
    Some(Timestamp {
        seconds: dt.timestamp(),
        nanos: i32::try_from(dt.timestamp_subsec_nanos()).unwrap_or(0),
    })
}

/// Proto `Timestamp` → domain `DateTime<Utc>` (for request fields).
#[must_use]
pub fn dt(ts: Option<Timestamp>) -> Option<DateTime<Utc>> {
    ts.and_then(|t| {
        Utc.timestamp_opt(t.seconds, u32::try_from(t.nanos).unwrap_or(0))
            .single()
    })
}

// ── Log stream ───────────────────────────────────────────────────────────────

/// Domain stream name (`"stdout"` / `"stderr"`) → the proto enum. Anything else
/// is `UNSPECIFIED` rather than a silent guess.
#[must_use]
pub fn log_stream_to_proto(name: &str) -> LogStream {
    match name {
        "stdout" => LogStream::Stdout,
        "stderr" => LogStream::Stderr,
        _ => LogStream::Unspecified,
    }
}

/// Proto enum → the domain stream name. `UNSPECIFIED` and unknown values fall
/// back to `"stdout"`, matching how an unlabelled line was treated before.
#[must_use]
pub fn log_stream_from_proto(raw: i32) -> &'static str {
    match LogStream::try_from(raw) {
        Ok(LogStream::Stderr) => "stderr",
        _ => "stdout",
    }
}

// ── Authz enum ⇄ catalog-key conversions ─────────────────────────────────────
// The proto `Permission` enum mirrors the backend `Permission` catalog: each
// value's proto name is `PERMISSION_` + the SCREAMING_SNAKE form of the
// camelCase key (`PERMISSION_RUN_PIPELINE` ⇄ `runPipeline`). buf lint requires
// the enum-name prefix; prost strips it from the Rust variant but NOT from
// `as_str_name()`, so both transforms handle it explicitly. The
// `permission_catalog_matches_proto` test guarantees the mapping is total.

/// The `as_str_name()` / `from_str_name()` prefix carried by every value of the
/// proto `Permission` enum.
const PERMISSION_PREFIX: &str = "PERMISSION_";

fn screaming_snake(camel: &str) -> String {
    let mut out = String::new();
    for ch in camel.chars() {
        if ch.is_ascii_uppercase() {
            out.push('_');
        }
        out.push(ch.to_ascii_uppercase());
    }
    out
}

fn camel_case(screaming: &str) -> String {
    let mut out = String::new();
    let mut upper_next = false;
    for ch in screaming.chars() {
        if ch == '_' {
            upper_next = true;
        } else if upper_next {
            out.push(ch);
            upper_next = false;
        } else {
            out.push(ch.to_ascii_lowercase());
        }
    }
    out
}

/// The catalog key for a proto `Permission` (e.g. `PERMISSION_RUN_PIPELINE` →
/// `"runPipeline"`). `None` for the `UNSPECIFIED` sentinel.
#[must_use]
pub fn permission_key(p: Permission) -> Option<String> {
    (p != Permission::Unspecified).then(|| {
        let name = p.as_str_name();
        camel_case(name.strip_prefix(PERMISSION_PREFIX).unwrap_or(name))
    })
}

/// The proto `Permission` for a catalog key, or `None` if the key is unknown.
#[must_use]
pub fn permission_from_key(key: &str) -> Option<Permission> {
    Permission::from_str_name(&format!("{PERMISSION_PREFIX}{}", screaming_snake(key)))
}

// ── Scope ────────────────────────────────────────────────────────────────────

/// Domain `ScopeKind` → the proto id-free discriminant.
#[must_use]
pub fn scope_kind_to_proto(kind: ScopeKind) -> ProtoScopeKind {
    match kind {
        ScopeKind::System => ProtoScopeKind::System,
        ScopeKind::Organization => ProtoScopeKind::Organization,
        ScopeKind::Project => ProtoScopeKind::Project,
    }
}

/// Proto `ScopeKind` → domain `ScopeKind`, erroring on unset/unknown.
pub fn scope_kind_from_proto(kind: i32) -> Result<ScopeKind, Status> {
    match ProtoScopeKind::try_from(kind) {
        Ok(ProtoScopeKind::System) => Ok(ScopeKind::System),
        Ok(ProtoScopeKind::Organization) => Ok(ScopeKind::Organization),
        Ok(ProtoScopeKind::Project) => Ok(ScopeKind::Project),
        Ok(ProtoScopeKind::Unspecified) | Err(_) => Err(Status::invalid_argument(
            "unknown or unspecified scope kind",
        )),
    }
}

/// Domain `Scope` → proto `ScopeRef`. The bound id lives inside its arm, so
/// there is no id to invent for `System`.
#[must_use]
pub fn scope_ref_to_proto(scope: &Scope) -> ScopeRef {
    let inner = match scope {
        Scope::System => scope_ref::Scope::System(scope_ref::System {}),
        Scope::Organization(id) => scope_ref::Scope::Organization(scope_ref::Organization {
            organization_id: wrap(id.to_string()),
        }),
        Scope::Project(id) => scope_ref::Scope::Project(scope_ref::Project {
            project_id: wrap(id.to_string()),
        }),
    };
    ScopeRef { scope: Some(inner) }
}

/// Proto `ScopeRef` → domain `Scope`. An absent oneof means the peer either sent
/// nothing or sent an arm added after this binary was built; both are rejected
/// rather than guessed at.
pub fn scope_ref_from_proto(scope: Option<ScopeRef>) -> Result<Scope, Status> {
    let scope = scope.ok_or_else(|| Status::invalid_argument("missing scope"))?;
    match scope.scope {
        Some(scope_ref::Scope::System(_)) => Ok(Scope::System),
        Some(scope_ref::Scope::Organization(o)) => Ok(Scope::Organization(OrganizationId::new(
            &required(o.organization_id, "scope.organization.organization_id")?,
        ))),
        Some(scope_ref::Scope::Project(p)) => Ok(Scope::Project(ProjectId::new(&required(
            p.project_id,
            "scope.project.project_id",
        )?))),
        None => Err(Status::invalid_argument(
            "scope is required (system, organization or project)",
        )),
    }
}

// ── Principal ────────────────────────────────────────────────────────────────

/// Domain `Principal` → proto `PrincipalRef`.
#[must_use]
pub fn principal_ref_to_proto(principal: &Principal) -> PrincipalRef {
    let inner = match principal {
        Principal::User(id) => principal_ref::Principal::User(principal_ref::User {
            user_id: wrap(id.to_string()),
        }),
        Principal::App(id) => principal_ref::Principal::App(principal_ref::App {
            app_id: wrap(id.to_string()),
        }),
    };
    PrincipalRef {
        principal: Some(inner),
    }
}

/// Proto `PrincipalRef` → domain `Principal`, rejecting an absent oneof.
pub fn principal_ref_from_proto(principal: Option<PrincipalRef>) -> Result<Principal, Status> {
    let principal = principal.ok_or_else(|| Status::invalid_argument("missing principal"))?;
    match principal.principal {
        Some(principal_ref::Principal::User(u)) => Ok(Principal::User(UserId::new(&required(
            u.user_id,
            "principal.user.user_id",
        )?))),
        Some(principal_ref::Principal::App(a)) => Ok(Principal::App(AppId::new(&required(
            a.app_id,
            "principal.app.app_id",
        )?))),
        None => Err(Status::invalid_argument(
            "principal is required (user or app)",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scylla_core::domain::value_objects::permission::PERMISSION_CATALOG;

    #[test]
    fn permission_catalog_matches_proto_enum() {
        // Every backend permission key round-trips through the proto enum. This
        // is the single guard that the hand-written proto enum stays a total
        // mirror of the code-owned catalog — and that the `PERMISSION_` prefix
        // is stripped and re-added consistently.
        for (key, _resource_type) in PERMISSION_CATALOG.iter() {
            let p = permission_from_key(key)
                .unwrap_or_else(|| panic!("no proto Permission for catalog key '{key}'"));
            assert_eq!(
                permission_key(p).as_deref(),
                Some(*key),
                "round-trip for {key}"
            );
        }
    }

    #[test]
    fn scope_ref_round_trips_every_arm() {
        // A ScopeRef must survive domain → proto → domain unchanged, including
        // System, which carries no id at all.
        for scope in [
            Scope::System,
            Scope::Organization(OrganizationId::new("org-1")),
            Scope::Project(ProjectId::new("proj-1")),
        ] {
            let round_tripped = scope_ref_from_proto(Some(scope_ref_to_proto(&scope))).unwrap();
            assert_eq!(format!("{round_tripped:?}"), format!("{scope:?}"));
        }
    }

    #[test]
    fn scope_ref_rejects_an_unset_union() {
        // An empty oneof means "arm added after this binary was built" — it must
        // be an error, never a silent default to System.
        assert!(scope_ref_from_proto(Some(ScopeRef { scope: None })).is_err());
        assert!(scope_ref_from_proto(None).is_err());
    }
}
