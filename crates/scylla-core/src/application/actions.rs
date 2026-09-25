//! The pipeline's authorizer is the access model's permission check. An adapter, not a blanket
//! impl: `Authorizer` and `PermissionService` are both foreign here (E0210).

use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::permission::Permission;
use async_trait::async_trait;
use scylla_auth::authz::PermissionService;
use scylla_extension::Authorizer;
use std::sync::Arc;

pub struct PermissionAuthorizer {
    permissions: Arc<dyn PermissionService>,
}

impl PermissionAuthorizer {
    #[must_use]
    pub fn new(permissions: Arc<dyn PermissionService>) -> Self {
        Self { permissions }
    }
}

#[async_trait]
impl Authorizer for PermissionAuthorizer {
    async fn authorize(&self, caller: &CallerContext, permission: Permission) -> DomainResult<()> {
        self.permissions.check(caller, permission).await
    }
}

/// The guard of a pass over many rows: no resource for Cedar to decide on, so the action is
/// `Authenticated` and its `Prepare` refuses every caller but an in-process service.
pub(crate) fn service_only(caller: &CallerContext) -> DomainResult<()> {
    match caller {
        CallerContext::Service(_) => Ok(()),
        _ => Err(DomainError::forbidden(
            "only an in-process service runs this pass",
        )),
    }
}
