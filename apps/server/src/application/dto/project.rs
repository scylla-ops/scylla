use crate::domain::entities::Project;
use crate::domain::value_objects::{
    Description, OrganizationId, PaginationMetadata, PaginationParams, ProjectId, ProjectName,
    UserId, UserProjectId, UserProjectRole, Username,
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct CreateProjectRequestDto {
    pub name: ProjectName,
    pub description: Option<Description>,
    pub organization_id: OrganizationId,
    pub creator_id: UserId,
}

#[derive(Debug, Clone)]
pub struct GetProjectRequestDto {
    pub project_id: ProjectId,
}

#[derive(Debug, Clone)]
pub struct UpdateProjectRequestDto {
    pub project_id: ProjectId,
    pub name: Option<ProjectName>,
    pub description: Option<Description>,
}

#[derive(Debug, Clone)]
pub struct ProjectResponseDto {
    pub id: ProjectId,
    pub name: ProjectName,
    pub description: Option<Description>,
    pub organization_id: OrganizationId,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Project> for ProjectResponseDto {
    fn from(project: Project) -> Self {
        Self {
            id: project.id().to_owned(),
            name: project.name().to_owned(),
            description: project.description().map(|d| d.to_owned()),
            organization_id: project.organization_id().to_owned(),
            is_active: project.is_active(),
            created_at: project.created_at(),
            updated_at: project.updated_at(),
        }
    }
}

impl From<&Project> for ProjectResponseDto {
    fn from(project: &Project) -> Self {
        Self {
            id: project.id().to_owned(),
            name: project.name().to_owned(),
            description: project.description().map(|d| d.to_owned()),
            organization_id: project.organization_id().to_owned(),
            is_active: project.is_active(),
            created_at: project.created_at(),
            updated_at: project.updated_at(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToggleProjectActiveRequestDto {
    pub project_id: ProjectId,
}

#[derive(Debug, Clone)]
pub struct ToggleProjectActiveResponseDto {}

#[derive(Debug, Clone)]
pub struct DeleteProjectRequestDto {
    pub project_id: ProjectId,
}

#[derive(Debug, Clone)]
pub struct DeleteProjectResponseDto {}

#[derive(Debug, Clone)]
pub struct ListProjectsRequestDto {
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ListProjectsResponseDto {
    pub projects: Vec<ProjectResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone)]
pub struct ListProjectUsersRequestDto {
    pub project_id: ProjectId,
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ProjectUserInfoResponseDto {
    pub user_id: UserId,
    pub username: Username,
    pub role: UserProjectRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ListProjectUsersResponseDto {
    pub users: Vec<ProjectUserInfoResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone)]
pub struct ListUserProjectsRequestDto {
    pub user_id: UserId,
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ListUserProjectsResponseDto {
    pub projects: Vec<ProjectResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone)]
pub struct AddUserToProjectRequestDto {
    pub user_id: UserId,
    pub project_id: ProjectId,
    pub role: UserProjectRole,
}

#[derive(Debug, Clone)]
pub struct AddUserToProjectResponseDto {
    pub relation_id: UserProjectId,
}

#[derive(Debug, Clone)]
pub struct RemoveUserFromProjectRequestDto {
    pub user_id: UserId,
    pub project_id: ProjectId,
}

#[derive(Debug, Clone)]
pub struct RemoveUserFromProjectResponseDto {}
