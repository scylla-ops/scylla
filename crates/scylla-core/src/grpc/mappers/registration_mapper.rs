//! Wire to command, command outcome to wire. The handler holds none of it.

use crate::application::signup::Signup;
use crate::grpc::convert::{Parse, required, valid};
use scylla_domain::domain::organization::OrganizationName;
use scylla_domain::domain::user::{Email, Password, Username};
use scylla_proto::registration::v1::SignupRequest;
use tonic::Status;

impl Parse for SignupRequest {
    type Into = Signup;

    fn parse(self) -> Result<Signup, Status> {
        Ok(Signup {
            username: valid(self.username, Username::new)?,
            email: valid(required(self.email, "email")?, Email::new)?,
            password: valid(self.password, Password::new)?,
            organization_name: valid(self.organization_name, OrganizationName::new)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grpc::convert::wrap;
    use tonic::Code;

    fn request() -> SignupRequest {
        SignupRequest {
            username: "founder".into(),
            email: wrap("founder@example.com"),
            password: "SecurePass123!".into(),
            organization_name: "Founders Inc".into(),
        }
    }

    #[test]
    fn a_signup_request_becomes_a_command_with_validated_fields() {
        let command = request().parse().unwrap();

        assert_eq!(command.username.as_str(), "founder");
        assert_eq!(command.email.as_str(), "founder@example.com");
        assert_eq!(command.organization_name.as_str(), "Founders Inc");
    }

    #[test]
    fn a_missing_email_is_an_invalid_argument() {
        let Err(err) = SignupRequest {
            email: None,
            ..request()
        }
        .parse() else {
            panic!("a missing email must not parse");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
        assert_eq!(err.message(), "missing email");
    }
}
