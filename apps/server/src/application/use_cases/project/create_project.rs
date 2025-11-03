use crate::application::dto::{CreateProjectRequestDto, ProjectResponseDto};
use crate::application::ports::RbacEnforcer;
use crate::domain::entities::{Project, UserProject};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::{ProjectRepository, UserProjectRepository};
use crate::domain::value_objects::UserProjectRole;
use crate::infrastructure::rbac::RoleMapper;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct CreateProjectUseCase<R, U, E>
where
    R: ProjectRepository + ?Sized,
    U: UserProjectRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    project_repo: Arc<R>,
    user_project_repo: Arc<U>,
    rbac_enforcer: Arc<E>,
}

impl<R, U, E> CreateProjectUseCase<R, U, E>
where
    R: ProjectRepository + ?Sized,
    U: UserProjectRepository + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    pub async fn execute(
        &self,
        request: CreateProjectRequestDto,
    ) -> DomainResult<ProjectResponseDto> {
        let project_draft =
            Project::create(request.name, request.description, request.organization_id)?;
        let created_project = self.project_repo.create(&project_draft).await?;

        // Add creator as owner of the project
        let owner_relation = UserProject::create(
            request.creator_id.clone(),
            created_project.id().to_owned(),
            UserProjectRole::owner(),
        )?;
        let _ = self.user_project_repo.create(&owner_relation).await?;

        let casbin_role = RoleMapper::project_role_to_casbin(&UserProjectRole::owner());
        self.rbac_enforcer
            .add_role_for_user(
                &request.creator_id,
                casbin_role,
                created_project.id().as_str(),
            )
            .await
            .map_err(|e| {
                crate::domain::errors::DomainError::internal(format!(
                    "Failed to assign RBAC role to project creator: {}",
                    e
                ))
            })?;

        Ok(ProjectResponseDto::from(created_project))
    }
}
