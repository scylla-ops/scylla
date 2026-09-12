//! Scylla's access model: the RBAC primitives (permissions, roles, scoped
//! grants, visibility), the ports the rest of the system authorizes through,
//! and the Cedar adapter that implements them.
//!
//! This crate sits below `scylla-core`: every use case is generic over
//! [`authz::PermissionService`] and friends, and the Postgres adapters in
//! `scylla-db` implement [`authz::RoleRepository`], [`authz::GrantRepository`],
//! [`authz::AuthzEntityProvider`] and [`audit::AuditLog`]. It depends only on
//! the domain kernel.
//!
//! `GrantUseCases` is the one piece of RBAC orchestration that does not live
//! here: revoking an App's grant also drops its live agent stream, which ties
//! it to the dispatch ports in `scylla-core`.

/// The domain model, re-exported from the [`scylla_domain`] kernel so that
/// `crate::domain::...` paths keep naming it from anywhere in this crate.
#[doc(no_inline)]
pub use scylla_domain::domain;

pub mod audit;
pub mod authz;
pub mod caller;
pub mod cedar;

pub use audit::{AuditDecision, AuditEntry, AuditLog, NoopAuditLog};
pub use authz::{PermissionService, PolicyControl, Visibility, VisibilityResolver};
pub use caller::{CallerContext, ServiceIdentity};
pub use cedar::CedarPermissionService;
