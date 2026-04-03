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

#[cfg(test)]
mod tests {
    use super::*;
    use scylla_core::domain::value_objects::user::{PasswordHash, Username};

    #[test]
    fn user_to_proto_maps_all_fields() {
        let user = User::create(
            Username::new("alice").unwrap(),
            PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap(),
        );

        let proto = user_to_proto(&user);
        assert_eq!(proto.username, "alice");
        assert!(proto.is_active);
        assert!(!proto.user_id.is_empty());
        assert!(!proto.created_at.is_empty());
        assert!(!proto.updated_at.is_empty());
    }
}
