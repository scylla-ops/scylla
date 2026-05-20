use crate::application::caller::CallerContext;
use crate::application::permission::policy::PolicyControl;
use crate::application::permission::service::PermissionService;
use crate::domain::entities::{OrganizationId, ProjectId, UserId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

/// The scope a grant is bound to. Maps to the `?resource` slot of the linked
/// Cedar template (e.g. `Project::"X"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantScope {
    Organization(OrganizationId),
    Project(ProjectId),
}

/// An explicit, scoped role assignment — "user U holds role R within scope S".
/// Each grant materialises as one linked Cedar template instance at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub id: String,
    pub user_id: UserId,
    pub role: RoleName,
    pub scope: GrantScope,
}

impl Grant {
    #[must_use]
    pub fn new(user_id: UserId, role: RoleName, scope: GrantScope) -> Self {
        Self {
            id: ulid::Ulid::new().to_string().to_lowercase(),
            user_id,
            role,
            scope,
        }
    }
}

/// Persistence for explicit scoped grants. Read at `CedarPermissionService`
/// construction to link template instances; mutated by `GrantUseCases`.
#[async_trait]
pub trait GrantRepository: Send + Sync {
    async fn list_all(&self) -> DomainResult<Vec<Grant>>;
    async fn create(&self, grant: &Grant) -> DomainResult<()>;
    async fn delete(&self, id: &str) -> DomainResult<()>;
}

/// Admin-only management of scoped grants. Every method is gated by
/// `Permission::ManageGrants` (admin/service in practice). A created or revoked
/// grant is applied live via [`PolicyControl::reload`], so it takes effect
/// immediately without a control-plane restart.
#[derive(Constructor)]
pub struct GrantUseCases<G: GrantRepository, PC: PolicyControl, PS: PermissionService> {
    grant_repo: Arc<G>,
    policy_control: Arc<PC>,
    permission_service: Arc<PS>,
}

impl<G: GrantRepository, PC: PolicyControl, PS: PermissionService> GrantUseCases<G, PC, PS> {
    #[instrument(skip(self, caller))]
    pub async fn list(&self, caller: &CallerContext) -> DomainResult<Vec<Grant>> {
        self.permission_service
            .check(caller, Permission::ManageGrants)
            .await?;
        self.grant_repo.list_all().await
    }

    #[instrument(skip(self, caller))]
    pub async fn grant(&self, caller: &CallerContext, grant: &Grant) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::ManageGrants)
            .await?;
        self.grant_repo.create(grant).await?;
        self.policy_control.reload().await
    }

    #[instrument(skip(self, caller))]
    pub async fn revoke(&self, caller: &CallerContext, id: &str) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::ManageGrants)
            .await?;
        self.grant_repo.delete(id).await?;
        self.policy_control.reload().await
    }
}
