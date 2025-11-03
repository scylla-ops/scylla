use crate::application::dto::{CreateOrganizationRequestDto, OrganizationResponseDto};
use crate::application::ports::RbacEnforcer;
use crate::domain::entities::{Organization, UserOrganization};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::{OrganizationRepository, UserOrganizationRepository};
use crate::domain::value_objects::UserOrganizationRole;
use crate::infrastructure::rbac::RoleMapper;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct CreateOrganizationUseCase<R, U, E>
where
    R: OrganizationRepository + ?Sized,
    U: UserOrganizationRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    org_repo: Arc<R>,
    user_org_repo: Arc<U>,
    rbac_enforcer: Arc<E>,
}

impl<R, U, E> CreateOrganizationUseCase<R, U, E>
where
    R: OrganizationRepository + ?Sized,
    U: UserOrganizationRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    pub async fn execute(
        &self,
        request: CreateOrganizationRequestDto,
    ) -> DomainResult<OrganizationResponseDto> {
        if self.org_repo.name_exists(&request.name).await? {
            return Err(DomainError::conflict(format!(
                "Organization with name '{}' already exists",
                request.name
            )));
        }

        let organization_draft = Organization::create(request.name, request.description)?;
        let created_organization = self.org_repo.create(&organization_draft).await?;

        // Add creator as owner of the organization
        let owner_relation = UserOrganization::create(
            request.creator_id.clone(),
            created_organization.id().to_owned(),
            UserOrganizationRole::owner(),
        )?;
        let _ = self.user_org_repo.create(&owner_relation).await?;

        let casbin_role = RoleMapper::org_role_to_casbin(&UserOrganizationRole::owner());
        self.rbac_enforcer
            .add_role_for_user(
                &request.creator_id,
                casbin_role,
                created_organization.id().as_str(),
            )
            .await
            .map_err(|e| {
                DomainError::internal(format!(
                    "Failed to assign RBAC role to organization creator: {}",
                    e
                ))
            })?;

        Ok(OrganizationResponseDto::from(created_organization))
    }
}
