use protocol::services::user::UserResponse;
use scylla_core::domain::entities::User;

pub fn user_to_proto(user: &User) -> UserResponse {
    UserResponse {
        user_id: user.id().to_string(),
        username: user.username().to_string(),
        is_active: user.is_active(),
        created_at: user.created_at().to_rfc3339(),
        updated_at: user.updated_at().to_rfc3339(),
    }
}
