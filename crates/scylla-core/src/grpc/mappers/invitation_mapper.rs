//! Wire to command, command outcome to wire. The handler holds none of it.

use crate::application::invitation::{CreateInvitation, ListInvitations};
use crate::grpc::convert::{Parse, id, optional, required, ts, valid, wrap};
use scylla_domain::domain::invitation::{
    Invitation as DomainInvitation, InvitationStatus as DomainInvitationStatus,
};
use scylla_domain::domain::role::RoleName;
use scylla_domain::domain::user::Email;
use scylla_proto::invitation::v1::{
    CreateInvitationRequest, Invitation, InvitationStatus, ListInvitationsRequest,
};
use tonic::Status;

fn status_to_proto(status: DomainInvitationStatus) -> InvitationStatus {
    match status {
        DomainInvitationStatus::Pending => InvitationStatus::Pending,
        DomainInvitationStatus::Accepted => InvitationStatus::Accepted,
        DomainInvitationStatus::Revoked => InvitationStatus::Revoked,
    }
}

pub fn invitation_to_proto(i: &DomainInvitation) -> Invitation {
    Invitation {
        invitation_id: wrap(i.id().to_string()),
        organization_id: wrap(i.organization_id().to_string()),
        email: wrap(i.email().as_str().to_string()),
        role: i.role().and_then(|r| wrap(r.as_str().to_string())),
        status: status_to_proto(i.status()) as i32,
        expires_at: ts(i.expires_at()),
    }
}

impl Parse for CreateInvitationRequest {
    type Into = CreateInvitation;

    fn parse(self) -> Result<CreateInvitation, Status> {
        Ok(CreateInvitation {
            organization_id: id(self.organization_id, "organization_id")?,
            email: valid(required(self.email, "email")?, Email::new)?,
            role: optional(self.role)
                .map(|r| valid(r, RoleName::new))
                .transpose()?,
        })
    }
}

impl Parse for ListInvitationsRequest {
    type Into = ListInvitations;

    fn parse(self) -> Result<ListInvitations, Status> {
        Ok(ListInvitations {
            organization_id: id(self.organization_id, "organization_id")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scylla_proto::common::v1 as common;
    use tonic::Code;

    #[test]
    fn a_create_request_becomes_a_command_with_validated_fields() {
        let command = CreateInvitationRequest {
            organization_id: wrap("org-1"),
            email: wrap("newbie@example.com"),
            role: wrap("organization-admin"),
        }
        .parse()
        .unwrap();

        assert_eq!(command.organization_id.as_str(), "org-1");
        assert_eq!(command.email.as_str(), "newbie@example.com");
        assert_eq!(
            command.role.as_ref().map(RoleName::as_str),
            Some("organization-admin")
        );
    }

    #[test]
    fn a_create_request_without_a_role_has_none() {
        let command = CreateInvitationRequest {
            organization_id: wrap("org-1"),
            email: wrap("newbie@example.com"),
            role: None,
        }
        .parse()
        .unwrap();

        assert!(command.role.is_none());
    }

    #[test]
    fn a_missing_email_is_an_invalid_argument() {
        let Err(err) = CreateInvitationRequest {
            organization_id: wrap("org-1"),
            email: None,
            role: None,
        }
        .parse() else {
            panic!("a missing email must not parse");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
        assert_eq!(err.message(), "missing email");
    }

    #[test]
    fn an_invalid_email_is_an_invalid_argument() {
        let Err(err) = CreateInvitationRequest {
            organization_id: wrap("org-1"),
            email: wrap("not-an-email"),
            role: None,
        }
        .parse() else {
            panic!("an invalid email must not parse");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn a_missing_organization_id_is_an_invalid_argument() {
        let err = ListInvitationsRequest {
            organization_id: None::<common::OrganizationId>,
        }
        .parse()
        .unwrap_err();

        assert_eq!(err.code(), Code::InvalidArgument);
        assert_eq!(err.message(), "missing organization_id");
    }
}
