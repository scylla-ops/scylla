//! Wire to command, command outcome to wire. The handler holds none of it.

use crate::application::auth::{Login, RevokeToken, ValidateToken};
use crate::grpc::convert::{Parse, valid};
use scylla_domain::domain::user::Password;
use scylla_proto::auth::v1::{LoginRequest, RevokeTokenRequest, ValidateTokenRequest};
use tonic::Status;

impl Parse for LoginRequest {
    type Into = Login;

    fn parse(self) -> Result<Login, Status> {
        Ok(Login {
            identifier: self.identifier,
            password: valid(self.password, Password::new)?,
        })
    }
}

parse!(ValidateTokenRequest => ValidateToken { token: copy });

impl Parse for RevokeTokenRequest {
    type Into = RevokeToken;

    fn parse(self) -> Result<RevokeToken, Status> {
        if self.token.is_empty() {
            return Err(Status::invalid_argument("Token cannot be empty"));
        }
        Ok(RevokeToken { token: self.token })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn a_login_request_keeps_the_identifier() {
        let command = LoginRequest {
            identifier: "kevin@example.com".into(),
            password: "SecurePass123!".into(),
        }
        .parse()
        .unwrap();

        assert_eq!(command.identifier, "kevin@example.com");
    }

    #[test]
    fn an_invalid_password_is_an_invalid_argument() {
        let Err(err) = LoginRequest {
            identifier: "kevin".into(),
            password: String::new(),
        }
        .parse() else {
            panic!("an empty password must not parse");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn an_empty_token_to_revoke_is_an_invalid_argument() {
        let Err(err) = RevokeTokenRequest {
            token: String::new(),
        }
        .parse() else {
            panic!("an empty token must not parse");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
        assert_eq!(err.message(), "Token cannot be empty");
    }
}
