pub mod commands;
pub mod repository;

pub use commands::{NewSignup, Signup};
pub use repository::SignupRepository;

use crate::application::{HashService, SessionRepository};
use crate::domain::errors::DomainResult;
use crate::domain::ids::{OrganizationId, UserId};
use crate::domain::organization::{Organization, OrganizationName};
use crate::domain::role::RoleName;
use crate::domain::user::User;
use derive_more::Constructor;
use scylla_auth::authz::{Grant, ORGANIZATION_ADMIN_ROLE, PolicyControl, Principal, Scope};
use std::sync::Arc;

pub struct SignupOutcome {
    pub token: String,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
}

/// A user, its own organization and the grant that makes it the admin: what a signup and a
/// first OAuth login create, in one transaction.
pub struct NewAccount {
    pub user: User,
    pub organization: Organization,
    pub grant: Grant,
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
            grant,
        })
    }
}

/// The signup's stage runners, one block per action in `commands.rs`. `Signup` is `Public`: the
/// caller has no account yet.
#[derive(Constructor)]
pub struct SignupUseCases {
    pub(super) signup_repo: Arc<dyn SignupRepository>,
    pub(super) session_repo: Arc<dyn SessionRepository>,
    pub(super) hash_service: Arc<dyn HashService>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
}

#[cfg(test)]
mod tests;
