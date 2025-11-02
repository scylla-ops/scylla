use crate::application::dto::{AddUserToOrganizationRequestDto, AddUserToOrganizationResponseDto};
use crate::application::ports::RbacEnforcer;
use crate::domain::entities::UserOrganization;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserOrganizationRepository;
use crate::infrastructure::rbac::RoleMapper;
use std::sync::Arc;

pub struct AddUserToOrganizationUseCase<R, E>
where
    R: UserOrganizationRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    user_org_repo: Arc<R>,
    rbac_enforcer: Arc<E>,
}

impl<R, E> AddUserToOrganizationUseCase<R, E>
where
    R: UserOrganizationRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    pub fn new(user_org_repo: Arc<R>, rbac_enforcer: Arc<E>) -> Self {
        Self {
            user_org_repo,
            rbac_enforcer,
        }
    }

    pub async fn execute(
        &self,
        request: AddUserToOrganizationRequestDto,
    ) -> DomainResult<AddUserToOrganizationResponseDto> {
        // Check if user is already in the organization
        if let Ok(_existing) = self
            .user_org_repo
            .find_by_user_and_organization(&request.user_id, &request.organization_id)
            .await
        {
            return Err(crate::domain::errors::DomainError::conflict(format!(
                "User '{}' is already a member of organization '{}'",
                request.user_id, request.organization_id
            )));
        }

        let user_org_draft = UserOrganization::create(
            request.user_id.clone(),
            request.organization_id.clone(),
            request.role.clone(),
        )?;

        let created_user_org = self.user_org_repo.create(&user_org_draft).await?;

        let casbin_role = RoleMapper::org_role_to_casbin(&request.role);

        let domain = request.organization_id.as_str();

        self.rbac_enforcer
            .add_role_for_user(&request.user_id, casbin_role, domain)
            .await
            .map_err(|e| {
                crate::domain::errors::DomainError::internal(format!(
                    "Failed to assign RBAC role to user: {}",
                    e
                ))
            })?;

        Ok(AddUserToOrganizationResponseDto {
            relation_id: created_user_org.id().to_owned(),
        })
    }
}
