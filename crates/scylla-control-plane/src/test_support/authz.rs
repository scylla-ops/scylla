//! Reusable `PermissionService` doubles for use-case tests.
//!
//! [`RecordingPermissionService`] allows everything and records each
//! `(caller, permission)` it was asked to check, so a test can assert a use
//! case authorized the *exact* permission on the *exact* resource before
//! acting. [`DenyingPermissionService`] refuses everything, so a test can prove
//! a use case checks authorization BEFORE touching repositories, ciphers, or
//! any other collaborator (pair it with panicking stubs).

use crate::application::authz::service::PermissionService;
use crate::application::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::Permission;
use async_trait::async_trait;
use std::sync::Mutex;

/// Allow-all `PermissionService` that records every check, in call order.
#[derive(Default)]
pub struct RecordingPermissionService {
    checks: Mutex<Vec<(CallerContext, Permission)>>,
}

impl RecordingPermissionService {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every `(caller, permission)` pair checked so far, in call order.
    #[must_use]
    pub fn checks(&self) -> Vec<(CallerContext, Permission)> {
        self.checks.lock().unwrap().clone()
    }

    /// Just the permissions checked so far, in call order.
    #[must_use]
    pub fn permissions(&self) -> Vec<Permission> {
        self.checks
            .lock()
            .unwrap()
            .iter()
            .map(|(_, p)| p.clone())
            .collect()
    }
}

#[async_trait]
impl PermissionService for RecordingPermissionService {
    async fn check(&self, caller: &CallerContext, permission: Permission) -> DomainResult<()> {
        self.checks
            .lock()
            .unwrap()
            .push((caller.clone(), permission));
        Ok(())
    }
}

/// Deny-all `PermissionService`: every check fails with `Forbidden`.
#[derive(Default)]
pub struct DenyingPermissionService;

impl DenyingPermissionService {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PermissionService for DenyingPermissionService {
    async fn check(&self, _caller: &CallerContext, _permission: Permission) -> DomainResult<()> {
        Err(DomainError::forbidden("denied by DenyingPermissionService"))
    }
}
