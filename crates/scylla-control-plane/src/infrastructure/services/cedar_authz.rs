//! Pure, `self`-free helpers for the Cedar adapter: entity-UID construction,
//! request-entity building, and audit-mapping.
//! Extracted from `cedar_permission_service.rs` so each concern is isolated and
//! independently readable; the service module keeps only the orchestration
//! (`check`, policy-set build/reload, the trait impls).

use crate::application::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::permission::ResourceRef;
use cedar_policy::EntityUid;
use std::collections::HashSet;
use std::str::FromStr;

pub(crate) fn euid(type_name: &str, id: &str) -> DomainResult<EntityUid> {
    EntityUid::from_str(&format!("{type_name}::\"{id}\""))
        .map_err(|e| DomainError::Internal(format!("cedar uid {type_name}::{id}: {e}")))
}

pub(crate) fn parent_set(parent: Option<&EntityUid>) -> HashSet<EntityUid> {
    parent.cloned().into_iter().collect()
}

pub(crate) fn resource_uid(resource: &ResourceRef) -> DomainResult<EntityUid> {
    match resource {
        ResourceRef::System => euid("Scylla::System", "root"),
        ResourceRef::User(id) => euid("Scylla::User", id.as_str()),
        ResourceRef::Organization(id) => euid("Scylla::Organization", id.as_str()),
        ResourceRef::Project(id) => euid("Scylla::Project", id.as_str()),
        ResourceRef::Pipeline(id) => euid("Scylla::Pipeline", id.as_str()),
        ResourceRef::Job(id) => euid("Scylla::Job", id.as_str()),
        ResourceRef::App(id) => euid("Scylla::App", id.as_str()),
    }
}

/// `(kind, id)` decomposition of a caller for the audit trail.
pub(crate) fn principal_parts(caller: &CallerContext) -> (&'static str, Option<String>) {
    match caller {
        CallerContext::User(id) => ("user", Some(id.as_str().to_string())),
        CallerContext::App(id) => ("app", Some(id.as_str().to_string())),
        CallerContext::Service(svc) => ("service", Some(svc.as_str().to_string())),
        CallerContext::Anonymous => ("anonymous", None),
    }
}

/// `(kind, id)` decomposition of a resource for the audit trail.
pub(crate) fn resource_parts(resource: &ResourceRef) -> (&'static str, Option<String>) {
    match resource {
        ResourceRef::System => ("system", None),
        ResourceRef::User(id) => ("user", Some(id.as_str().to_string())),
        ResourceRef::Organization(id) => ("organization", Some(id.as_str().to_string())),
        ResourceRef::Project(id) => ("project", Some(id.as_str().to_string())),
        ResourceRef::Pipeline(id) => ("pipeline", Some(id.as_str().to_string())),
        ResourceRef::Job(id) => ("job", Some(id.as_str().to_string())),
        ResourceRef::App(id) => ("app", Some(id.as_str().to_string())),
    }
}
