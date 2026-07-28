use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::entities::User;
use scylla_protocol::common::v1 as common;
use scylla_protocol::user::v1::User as ProtoUser;

pub fn user_to_proto(user: &User) -> ProtoUser {
    ProtoUser {
        user_id: wrap(user.id().to_string()),
        username: user.username().to_string(),
        is_active: user.is_active(),
        created_at: ts(user.created_at()),
        updated_at: ts(user.updated_at()),
        email: user.email().map(|e| common::Email {
            value: e.as_str().to_string(),
        }),
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
            None,
            PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap(),
        );

        let proto = user_to_proto(&user);
        assert_eq!(proto.username, "alice");
        assert!(proto.is_active);
        assert!(!proto.user_id.unwrap().value.is_empty());
        assert!(proto.created_at.is_some());
        assert!(proto.updated_at.is_some());
    }
}
