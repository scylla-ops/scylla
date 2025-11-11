use crate::application::dto::UserResponseDto;
use protocol::services::user::UserResponse;

impl From<UserResponseDto> for UserResponse {
    fn from(dto: UserResponseDto) -> Self {
        UserResponse {
            user_id: dto.id.to_string(),
            username: dto.username.to_string(),
            created_at: dto.created_at.to_rfc3339(),
            updated_at: dto.updated_at.to_rfc3339(),
            is_active: dto.is_active,
        }
    }
}
