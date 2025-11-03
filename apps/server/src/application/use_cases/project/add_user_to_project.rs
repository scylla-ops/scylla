use crate::application::dto::{AddUserToProjectRequestDto, AddUserToProjectResponseDto};
use crate::application::ports::RbacEnforcer;
use crate::domain::entities::UserProject;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::UserProjectRepository;
use crate::infrastructure::rbac::RoleMapper;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct AddUserToProjectUseCase<R, E>
where
    R: UserProjectRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    user_project_repo: Arc<R>,
    rbac_enforcer: Arc<E>,
}

impl<R, E> AddUserToProjectUseCase<R, E>
where
    R: UserProjectRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    pub async fn execute(
        &self,
        request: AddUserToProjectRequestDto,
    ) -> DomainResult<AddUserToProjectResponseDto> {
        // Check if user is already in the project
        if let Ok(_existing) = self
            .user_project_repo
            .find_by_user_and_project(&request.user_id, &request.project_id)
            .await
        {
            return Err(crate::domain::errors::DomainError::conflict(format!(
                "User '{}' is already a member of project '{}'",
                request.user_id, request.project_id
            )));
        }

        let relation = UserProject::create(
            request.user_id.clone(),
            request.project_id.clone(),
            request.role.clone(),
        )?;
        let created = self.user_project_repo.create(&relation).await?;

        let casbin_role = RoleMapper::project_role_to_casbin(&request.role);

        let domain = request.project_id.as_str();

        self.rbac_enforcer
            .add_role_for_user(&request.user_id, casbin_role, domain)
            .await
            .map_err(|e| {
                crate::domain::errors::DomainError::internal(format!(
                    "Failed to assign RBAC role to user: {}",
                    e
                ))
            })?;

        Ok(AddUserToProjectResponseDto {
            relation_id: created.id().to_owned(),
        })
    }
}
