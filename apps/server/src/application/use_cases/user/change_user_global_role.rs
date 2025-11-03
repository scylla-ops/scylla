use crate::application::dto::{ChangeUserGlobalRoleRequestDto, ChangeUserGlobalRoleResponseDto};
use crate::application::ports::RbacEnforcer;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::UserRepository;
use crate::infrastructure::rbac::RoleMapper;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ChangeUserGlobalRoleUseCase<R, E>
where
    R: UserRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    user_repo: Arc<R>,
    rbac_enforcer: Arc<E>,
}

impl<R, E> ChangeUserGlobalRoleUseCase<R, E>
where
    R: UserRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    pub async fn execute(
        &self,
        request: ChangeUserGlobalRoleRequestDto,
    ) -> DomainResult<ChangeUserGlobalRoleResponseDto> {
        // verify target user exists
        self.user_repo
            .find_by_id(&request.user_id)
            .await
            .map_err(|_| DomainError::not_found("User", request.user_id.to_string()))?;

        // authorization check: only admins can change global roles
        // exception: if no admins exist, allow anyone to create the first admin (bootstrap)
        let admin_users = self.rbac_enforcer.get_users_for_role("admin", "*").await?;

        // if trying to set admin role and no admins exist, allow it (bootstrap mode)
        // otherwise, check if caller has admin permissions
        let is_bootstrap = admin_users.is_empty() && request.new_role.is_admin();

        if !is_bootstrap {
            // check that caller has permission to update users (only admins can)
            // note: caller_id should be passed from the handler via request context
            // for now, we'll add it to the DTO
            if let Some(caller_id) = &request.caller_id {
                let has_permission = self
                    .rbac_enforcer
                    .enforce(caller_id, "*", "users", "update")
                    .await?;

                if !has_permission {
                    return Err(DomainError::unauthorized(
                        "Only administrators can change global user roles",
                    ));
                }
            } else {
                // if no caller_id provided, reject (should come from authenticated context)
                return Err(DomainError::unauthorized(
                    "Authentication required to change user roles",
                ));
            }
        }

        // get current roles for the user in global domain
        let current_roles = self
            .rbac_enforcer
            .get_roles_for_user(&request.user_id, "*")
            .await?;

        // remove all existing global roles (should be only one: "admin" or "user")
        for role in &current_roles {
            self.rbac_enforcer
                .remove_role_for_user(&request.user_id, role, "*")
                .await
                .map_err(|e| {
                    DomainError::internal(format!("Failed to remove existing global role: {}", e))
                })?;
        }

        // add the new role
        let new_casbin_role = RoleMapper::global_role_to_casbin(&request.new_role);
        self.rbac_enforcer
            .add_role_for_user(&request.user_id, new_casbin_role, "*")
            .await
            .map_err(|e| {
                DomainError::internal(format!("Failed to assign new global role: {}", e))
            })?;

        Ok(ChangeUserGlobalRoleResponseDto {
            user_id: request.user_id,
            new_role: request.new_role,
        })
    }
}
