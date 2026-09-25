use crate::application::user::{CreateUser, DeleteUser, GetUser, ListUsers, UpdateUser};
use crate::grpc::convert::{Parse, id, optional, ts, valid, wrap};
use scylla_domain::domain::user::{Email, Password, User, Username};
use scylla_proto::common::v1 as common;
use scylla_proto::user::v1::{
    CreateUserRequest, DeleteUserRequest, GetUserRequest, ListUsersRequest, ListUsersResponse,
    UpdateUserRequest, User as ProtoUser,
};
use tonic::Status;

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

impl Parse for CreateUserRequest {
    type Into = CreateUser;

    fn parse(self) -> Result<CreateUser, Status> {
        Ok(CreateUser {
            username: valid(self.username, Username::new)?,
            password: valid(self.password, Password::new)?,
            email: optional(self.email)
                .map(|e| valid(e, Email::new))
                .transpose()?,
        })
    }
}

parse!(GetUserRequest => GetUser { id: id(user_id) });

impl Parse for UpdateUserRequest {
    type Into = UpdateUser;

    fn parse(self) -> Result<UpdateUser, Status> {
        Ok(UpdateUser {
            id: id(self.user_id, "user_id")?,
            username: self.username.map(|u| valid(u, Username::new)).transpose()?,
        })
    }
}

parse!(DeleteUserRequest => DeleteUser { id: id(user_id) });
parse!(ListUsersRequest => ListUsers { pagination: page });
page_response!(User => users: user_to_proto; ListUsersResponse);

#[cfg(test)]
mod tests {
    use super::*;
    use scylla_domain::domain::user::PasswordHash;
    use tonic::Code;

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

    #[test]
    fn a_create_request_becomes_a_command_with_validated_fields() {
        let command = CreateUserRequest {
            username: "alice".into(),
            password: "SecurePass123!".into(),
            email: wrap("alice@example.com"),
        }
        .parse()
        .unwrap();

        assert_eq!(command.username.as_str(), "alice");
        assert_eq!(command.password.as_str(), "SecurePass123!");
        assert_eq!(
            command.email.as_ref().map(Email::as_str),
            Some("alice@example.com")
        );
    }

    #[test]
    fn a_missing_id_is_an_invalid_argument() {
        let err = GetUserRequest {
            user_id: None::<common::UserId>,
        }
        .parse()
        .unwrap_err();

        assert_eq!(err.code(), Code::InvalidArgument);
        assert_eq!(err.message(), "missing user_id");
    }

    #[test]
    fn a_domain_validation_failure_is_an_invalid_argument() {
        let err = CreateUserRequest {
            username: "alice".into(),
            password: "short".into(),
            email: None,
        }
        .parse()
        .unwrap_err();

        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn an_update_without_a_username_changes_nothing() {
        let command = UpdateUserRequest {
            user_id: wrap("user-1"),
            username: None,
        }
        .parse()
        .unwrap();

        assert_eq!(command.id.as_str(), "user-1");
        assert!(command.username.is_none());
    }
}
