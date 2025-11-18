use crate::application::dto::{ProjectResponseDto, ProjectUserInfoResponseDto};
use protocol::services::project::{ProjectResponse, ProjectUserInfoResponse};

impl From<ProjectResponseDto> for ProjectResponse {
    fn from(dto: ProjectResponseDto) -> Self {
        ProjectResponse {
            project_id: dto.id.to_string(),
            name: dto.name.to_string(),
            description: dto.description.map(|d| d.to_string()).unwrap_or_default(),
            organization_id: dto.organization_id.to_string(),
            is_active: dto.is_active,
            created_at: dto.created_at.to_rfc3339(),
            updated_at: dto.updated_at.to_rfc3339(),
        }
    }
}

impl From<ProjectUserInfoResponseDto> for ProjectUserInfoResponse {
    fn from(dto: ProjectUserInfoResponseDto) -> Self {
        ProjectUserInfoResponse {
            user_id: dto.user_id.to_string(),
            username: dto.username.to_string(),
            role: dto.role.to_string(),
            joined_at: dto.joined_at.to_rfc3339(),
        }
    }
}
