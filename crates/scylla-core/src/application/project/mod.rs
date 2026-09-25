pub mod commands;
pub mod queries;
pub mod repository;

pub use commands::{CreateProject, DeleteProject, NewProject, SetProjectActive, UpdateProject};
pub use queries::{
    GetProject, ListOrganizationProjects, ListProjectMembers, ListProjects, ListUserProjects,
};
pub use repository::ProjectRepository;

use crate::application::UserRepository;
use derive_more::Constructor;
use scylla_auth::authz::{PermissionService, PolicyControl, VisibilityResolver};
use std::sync::Arc;

/// The project aggregate's stage runners, one block per action in `commands.rs` and
/// `queries.rs`. It has no method of its own; `Actions::run` drives it. `permission_service`
/// serves one scoping decision in `queries.rs`, never a gate.
#[derive(Constructor)]
pub struct ProjectUseCases {
    pub(super) project_repo: Arc<dyn ProjectRepository>,
    pub(super) user_repo: Arc<dyn UserRepository>,
    pub(super) permission_service: Arc<dyn PermissionService>,
    pub(super) visibility: Arc<dyn VisibilityResolver>,
    pub(super) policy_control: Arc<dyn PolicyControl>,
}

#[cfg(test)]
mod tests;
