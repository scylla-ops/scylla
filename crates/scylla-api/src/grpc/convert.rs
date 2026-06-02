//! Conversions between the strongly-typed proto wrappers from `common.proto`
//! (ids, email) plus `google.protobuf.Timestamp`, and the plain `String` ids /
//! `chrono` timestamps the domain uses. Centralised here so each mapper and
//! handler site stays a one-liner.

use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;
use scylla_core::application::{Scope, ScopeKind};
use scylla_protocol::services::common;
use scylla_protocol::services::permission::{Permission, ResourceType, Scope as ProtoScope};
use tonic::Status;

/// A proto wrapper message (`common.*Id` / `common.Email`) — a single `value`.
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

// ── Authz enum ⇄ catalog-key conversions ─────────────────────────────────────
// The proto `Permission` enum mirrors the backend `Permission` catalog: each
// value's proto name is the SCREAMING_SNAKE form of the camelCase key
// (`RUN_PIPELINE` ⇄ `runPipeline`). These transforms bridge the two; the
// `permission_catalog_matches_proto` test guarantees the mapping is total.

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

/// The catalog key for a proto `Permission` (e.g. `RUN_PIPELINE` → `"runPipeline"`).
/// `None` for the `UNSPECIFIED` sentinel.
#[must_use]
pub fn permission_key(p: Permission) -> Option<String> {
    (p != Permission::Unspecified).then(|| camel_case(p.as_str_name()))
}

/// The proto `Permission` for a catalog key, or `None` if the key is unknown.
#[must_use]
pub fn permission_from_key(key: &str) -> Option<Permission> {
    Permission::from_str_name(&screaming_snake(key))
}

/// The proto `ResourceType` for a resource-type tag (e.g. `"pipeline"` → `PIPELINE`).
#[must_use]
pub fn resource_type_from_tag(tag: &str) -> ResourceType {
    ResourceType::from_str_name(&format!("RESOURCE_TYPE_{}", tag.to_ascii_uppercase()))
        .unwrap_or(ResourceType::Unspecified)
}

/// Domain `ScopeKind` → proto `Scope`.
#[must_use]
pub fn scope_kind_to_proto(kind: ScopeKind) -> ProtoScope {
    match kind {
        ScopeKind::System => ProtoScope::System,
        ScopeKind::Organization => ProtoScope::Organization,
        ScopeKind::Project => ProtoScope::Project,
    }
}

/// Domain `Scope` (with id) → proto `Scope` discriminant + the bound id
/// (empty for `System`).
#[must_use]
pub fn scope_to_proto(scope: &Scope) -> (ProtoScope, String) {
    match scope {
        Scope::System => (ProtoScope::System, String::new()),
        Scope::Organization(id) => (ProtoScope::Organization, id.to_string()),
        Scope::Project(id) => (ProtoScope::Project, id.to_string()),
    }
}

/// Proto `Scope` discriminant → domain `ScopeKind`, erroring on unset/unknown.
pub fn scope_kind_from_proto(kind: i32) -> Result<ScopeKind, Status> {
    match ProtoScope::try_from(kind) {
        Ok(ProtoScope::System) => Ok(ScopeKind::System),
        Ok(ProtoScope::Organization) => Ok(ScopeKind::Organization),
        Ok(ProtoScope::Project) => Ok(ScopeKind::Project),
        Ok(ProtoScope::Unspecified) | Err(_) => {
            Err(Status::invalid_argument("unknown or unspecified scope"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scylla_core::domain::value_objects::permission::{PERMISSION_CATALOG, RESOURCE_TYPES};

    #[test]
    fn permission_catalog_matches_proto_enum() {
        // Every backend permission key round-trips through the proto enum, and
        // every resource type maps to a concrete proto ResourceType. This is the
        // single guard that the hand-written proto enum stays in sync with the
        // code-owned catalog.
        for (key, rt) in PERMISSION_CATALOG {
            let p = permission_from_key(key)
                .unwrap_or_else(|| panic!("no proto Permission for catalog key '{key}'"));
            assert_eq!(
                permission_key(p).as_deref(),
                Some(*key),
                "round-trip for {key}"
            );
            assert_ne!(
                resource_type_from_tag(rt),
                ResourceType::Unspecified,
                "no proto ResourceType for '{rt}'"
            );
        }
        for rt in RESOURCE_TYPES {
            assert_ne!(resource_type_from_tag(rt), ResourceType::Unspecified);
        }
    }
}
