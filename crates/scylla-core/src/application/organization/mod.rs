pub mod commands;
pub mod queries;
pub mod repository;

pub use commands::{
    CreateOrganization, DeleteOrganization, NewOrganization, SetOrganizationActive,
    UpdateOrganization,
};
pub use queries::{
    GetOrganization, ListOrganizationMembers, ListOrganizations, ListUserOrganizations,
};
pub use repository::OrganizationRepository;

use crate::application::UserRepository;
use derive_more::Constructor;
use scylla_auth::authz::PolicyControl;
use std::sync::Arc;

/// The organization aggregate's stage runners, one block per action in `commands.rs` and
/// `queries.rs`. It has no method of its own; `Actions::run` drives it.
#[derive(Constructor)]
pub struct OrganizationUseCases {
    pub(super) org_repo: Arc<dyn OrganizationRepository>,
    pub(super) user_repo: Arc<dyn UserRepository>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
}

#[cfg(test)]
mod tests;
