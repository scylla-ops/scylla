//! The Cedar-backed authorization adapter: `CedarPermissionService` implements
//! `PermissionService`, `VisibilityResolver` and `PolicyControl` over the
//! schema and policies embedded from this directory.

mod authz;
pub mod permission_service;

pub use permission_service::CedarPermissionService;
