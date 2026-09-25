//! Wire to command, command outcome to wire. The handler holds none of it.

use crate::application::grant::{CreateGrant, ListGrants, RevokeAllAccess, RevokeGrant};
use crate::grpc::convert::{
    Parse, principal_ref_from_proto, principal_ref_to_proto, required, scope_kind_to_proto,
    scope_ref_from_proto, scope_ref_to_proto, valid, wrap,
};
use scylla_auth::authz::{Grant, GrantableRole, RoleKind};
use scylla_domain::domain::role::RoleName;
use scylla_proto::authz::v1::{
    CreateGrantRequest, Grant as ProtoGrant, GrantableRole as ProtoGrantableRole,
    ListGrantsRequest, RevokeAllAccessRequest, RevokeGrantRequest, RoleKind as ProtoRoleKind,
};
use tonic::Status;

pub fn grant_to_proto(g: &Grant) -> ProtoGrant {
    ProtoGrant {
        grant_id: wrap(g.id.clone()),
        principal: Some(principal_ref_to_proto(&g.principal)),
        scope: Some(scope_ref_to_proto(&g.scope)),
        role: wrap(g.role.to_string()),
    }
}

pub fn grantable_role_to_proto(r: &GrantableRole) -> ProtoGrantableRole {
    ProtoGrantableRole {
        role_id: wrap(r.name),
        scope_kind: scope_kind_to_proto(r.scope) as i32,
        kind: role_kind_to_proto(r.kind) as i32,
        description: r.description.to_string(),
    }
}

fn role_kind_to_proto(kind: RoleKind) -> ProtoRoleKind {
    match kind {
        RoleKind::Admin => ProtoRoleKind::Admin,
        RoleKind::Member => ProtoRoleKind::Member,
        RoleKind::Agent => ProtoRoleKind::Agent,
    }
}

impl Parse for CreateGrantRequest {
    type Into = CreateGrant;

    fn parse(self) -> Result<CreateGrant, Status> {
        Ok(CreateGrant {
            principal: principal_ref_from_proto(self.principal)?,
            scope: scope_ref_from_proto(self.scope)?,
            role: valid(required(self.role, "role")?, RoleName::new)?,
        })
    }
}

impl Parse for RevokeAllAccessRequest {
    type Into = RevokeAllAccess;

    fn parse(self) -> Result<RevokeAllAccess, Status> {
        Ok(RevokeAllAccess {
            principal: principal_ref_from_proto(self.principal)?,
            scope: scope_ref_from_proto(self.scope)?,
        })
    }
}

parse!(RevokeGrantRequest => RevokeGrant { id: id(grant_id) });

impl Parse for ListGrantsRequest {
    type Into = ListGrants;

    fn parse(self) -> Result<ListGrants, Status> {
        Ok(ListGrants {
            scope: self
                .scope
                .map(|scope| scope_ref_from_proto(Some(scope)))
                .transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scylla_auth::authz::{Principal, Scope};
    use scylla_domain::domain::ids::{OrganizationId, UserId};
    use scylla_proto::authz::v1::ScopeRef;
    use tonic::Code;

    fn org_scope() -> Scope {
        Scope::Organization(OrganizationId::new("org-1"))
    }

    fn alice() -> Principal {
        Principal::User(UserId::new("alice"))
    }

    #[test]
    fn a_create_request_becomes_a_command_with_its_principal_scope_and_role() {
        let command = CreateGrantRequest {
            principal: Some(principal_ref_to_proto(&alice())),
            scope: Some(scope_ref_to_proto(&org_scope())),
            role: wrap("organization-admin"),
        }
        .parse()
        .unwrap();

        assert_eq!(command.principal, alice());
        assert_eq!(command.scope, org_scope());
        assert_eq!(command.role.as_str(), "organization-admin");
    }

    #[test]
    fn a_create_request_without_a_role_is_an_invalid_argument() {
        let Err(err) = CreateGrantRequest {
            principal: Some(principal_ref_to_proto(&alice())),
            scope: Some(scope_ref_to_proto(&org_scope())),
            role: None,
        }
        .parse() else {
            panic!("a missing role must be refused");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn a_revoke_all_request_needs_a_scope() {
        let Err(err) = RevokeAllAccessRequest {
            principal: Some(principal_ref_to_proto(&alice())),
            scope: None,
        }
        .parse() else {
            panic!("a missing scope must be refused");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn a_list_request_keeps_its_optional_scope() {
        assert!(
            ListGrantsRequest { scope: None }
                .parse()
                .unwrap()
                .scope
                .is_none()
        );
        assert_eq!(
            ListGrantsRequest {
                scope: Some(scope_ref_to_proto(&org_scope())),
            }
            .parse()
            .unwrap()
            .scope,
            Some(org_scope())
        );
    }

    #[test]
    fn a_list_request_with_an_unset_scope_union_is_an_invalid_argument() {
        let Err(err) = ListGrantsRequest {
            scope: Some(ScopeRef { scope: None }),
        }
        .parse() else {
            panic!("an unset scope union must be refused");
        };

        assert_eq!(err.code(), Code::InvalidArgument);
    }
}
