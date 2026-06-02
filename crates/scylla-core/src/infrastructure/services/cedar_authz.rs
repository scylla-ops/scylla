//! Pure, `self`-free helpers for the Cedar adapter: entity-UID construction,
//! request-entity building, the anti-lockout `forbid` guard, and audit-mapping.
//! Extracted from `cedar_permission_service.rs` so each concern is isolated and
//! independently readable; the service module keeps only the orchestration
//! (`check`, policy-set build/reload, the trait impls).

use crate::application::authz::entity_provider::PrincipalAuthz;
use crate::application::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::ResourceRef;
use cedar_policy::{ActionConstraint, Entity, EntityUid, Policy, RestrictedExpression};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub(crate) fn euid(type_name: &str, id: &str) -> DomainResult<EntityUid> {
    EntityUid::from_str(&format!("{type_name}::\"{id}\""))
        .map_err(|e| DomainError::Internal(format!("cedar uid {type_name}::{id}: {e}")))
}

/// Actions a runtime `forbid` may never deny — denying any of these could lock
/// an admin out of fixing policies (the recovery path itself). A forbid that
/// touches one of these, or whose action scope is unconstrained, is rejected on
/// write and skipped on load.
pub(crate) const GUARDED_ACTIONS: &[&str] = &["managePolicies", "manageGrants"];

/// A runtime `forbid` is safe only if its action scope is concrete and excludes
/// the guarded admin actions. An unconstrained (`Any`) action is a catch-all
/// that would deny everyone — including recovery — so it is never allowed.
pub(crate) fn forbid_is_safe(policy: &Policy) -> bool {
    match policy.action_constraint() {
        ActionConstraint::Any => false,
        ActionConstraint::Eq(uid) => !is_guarded_action(&uid),
        ActionConstraint::In(uids) => !uids.iter().any(is_guarded_action),
    }
}

pub(crate) fn is_guarded_action(uid: &EntityUid) -> bool {
    GUARDED_ACTIONS
        .iter()
        .filter_map(|a| euid("Scylla::Action", a).ok())
        .any(|guarded| &guarded == uid)
}

pub(crate) fn parent_set(parent: Option<&EntityUid>) -> HashSet<EntityUid> {
    parent.cloned().into_iter().collect()
}

pub(crate) fn uid_set<T: AsRef<str>>(
    type_name: &str,
    ids: &[T],
) -> DomainResult<RestrictedExpression> {
    let mut exprs = Vec::with_capacity(ids.len());
    for id in ids {
        exprs.push(RestrictedExpression::new_entity_uid(euid(
            type_name,
            id.as_ref(),
        )?));
    }
    Ok(RestrictedExpression::new_set(exprs))
}

pub(crate) fn user_entity(uid: EntityUid, authz: &PrincipalAuthz) -> DomainResult<Entity> {
    // No role-membership parents anymore: global authority is a System-scoped
    // grant (linked template instance), not entity membership. The only parent is
    // the System root, so a User (as a *resource*) is reachable by a system-admin
    // grant's `resource in System`. ABAC org/project memberships are attrs.
    let parents = HashSet::from([euid("Scylla::System", "root")?]);
    let attrs = HashMap::from([
        (
            "memberOrgs".to_string(),
            uid_set("Scylla::Organization", &authz.member_orgs)?,
        ),
        (
            "memberProjects".to_string(),
            uid_set("Scylla::Project", &authz.member_projects)?,
        ),
    ]);
    Entity::new(uid, attrs, parents)
        .map_err(|e| DomainError::Internal(format!("cedar user entity: {e}")))
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
