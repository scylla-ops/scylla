pub mod repository;

pub use repository::SignupRepository;

use crate::domain::errors::DomainResult;
use crate::domain::organization::{Organization, OrganizationName};
use crate::domain::role::RoleName;
use crate::domain::user::User;
use scylla_auth::authz::{Grant, ORGANIZATION_ADMIN_ROLE, Principal, Scope};

/// A user, its own organization and the grants of the account, the first of which makes the
/// user its admin: what the sign-up of an edition creates in one transaction.
pub struct NewAccount {
    pub user: User,
    pub organization: Organization,
    pub grants: Vec<Grant>,
}

impl NewAccount {
    pub fn new(user: User, organization_name: OrganizationName) -> DomainResult<Self> {
        let organization = Organization::create(organization_name, None)?;
        let grant = Grant::new(
            Principal::User(user.id().clone()),
            RoleName::new(ORGANIZATION_ADMIN_ROLE)?,
            Scope::Organization(organization.id().clone()),
        );
        Ok(Self {
            user,
            organization,
            grants: vec![grant],
        })
    }

    /// One more grant for the user, written in the same transaction.
    #[must_use]
    pub fn with_grant(mut self, role: RoleName, scope: Scope) -> Self {
        self.grants.push(Grant::new(
            Principal::User(self.user.id().clone()),
            role,
            scope,
        ));
        self
    }
}
