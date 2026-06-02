//! Conversions between the strongly-typed proto wrappers from `common.proto`
//! (ids, email) plus `google.protobuf.Timestamp`, and the plain `String` ids /
//! `chrono` timestamps the domain uses. Centralised here so each mapper and
//! handler site stays a one-liner.

use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;
use scylla_protocol::services::common;
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
