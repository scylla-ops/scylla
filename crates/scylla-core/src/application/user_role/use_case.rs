use crate::application::caller::CallerContext;
use crate::application::permission::service::PermissionService;
use crate::application::user_role::repository::UserRoleRepository;
use crate::domain::entities::UserId;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::role::name::RoleName;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

/// Admin-only management of user ↔ role assignments. Every method is gated by
/// [`Permission::ManageRoles`]. Role membership is materialised live on every
/// authorization check (read from `user_roles`), so an assignment or revocation
/// takes effect immediately — no policy-set reload required (unlike grants).
#[derive(Constructor)]
pub struct UserRoleUseCases<URR: UserRoleRepository, PS: PermissionService> {
    user_role_repo: Arc<URR>,
    permission_service: Arc<PS>,
}

impl<URR: UserRoleRepository, PS: PermissionService> UserRoleUseCases<URR, PS> {
    #[instrument(skip(self, caller))]
    pub async fn list_roles(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
    ) -> DomainResult<Vec<RoleName>> {
        self.permission_service
            .check(caller, Permission::ManageRoles)
            .await?;
        self.user_role_repo.list_roles_for_user(user_id).await
    }

    #[instrument(skip(self, caller))]
    pub async fn assign(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
        role: &RoleName,
    ) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::ManageRoles)
            .await?;
        self.user_role_repo.assign(user_id, role).await
    }

    #[instrument(skip(self, caller))]
    pub async fn revoke(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
        role: &RoleName,
    ) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::ManageRoles)
            .await?;
        self.user_role_repo.revoke(user_id, role).await
    }
}
