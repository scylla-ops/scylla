use crate::application::dto::{OrganizationResponseDto, UserInfoDto};
use protocol::services::organization::{OrganizationResponse, OrganizationUserInfoResponse};

impl From<OrganizationResponseDto> for OrganizationResponse {
    fn from(dto: OrganizationResponseDto) -> Self {
        OrganizationResponse {
            organization_id: dto.id.to_string(),
            name: dto.name.to_string(),
            description: dto.description.map(|d| d.to_string()).unwrap_or_default(),
            is_active: dto.is_active,
            created_at: dto.created_at.to_rfc3339(),
            updated_at: dto.updated_at.to_rfc3339(),
        }
    }
}

impl From<UserInfoDto> for OrganizationUserInfoResponse {
    fn from(dto: UserInfoDto) -> Self {
        OrganizationUserInfoResponse {
            user_id: dto.user_id.to_string(),
            username: dto.username,
            role: dto.role.to_string(),
            joined_at: dto.joined_at.to_rfc3339(),
        }
    }
}
