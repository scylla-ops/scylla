pub mod error;
pub mod extract;
pub mod openapi;
pub mod response;
pub mod routes;

use scylla_core::application::{
    AuthUseCases, JobUseCases, OrganizationUseCases, PermissionUseCases, PipelineUseCases,
    ProjectUseCases, UserUseCases,
};
use scylla_core::infrastructure::{
    Argon2HashService, CasbinPermissionService, SurrealJobRepository,
    SurrealOrganizationRepository, SurrealPipelineRepository, SurrealProjectRepository,
    SurrealSessionRepository, SurrealUserOrganizationRepository, SurrealUserProjectRepository,
    SurrealUserRepository,
};
use std::sync::Arc;

pub type ConcreteAuthUc =
    AuthUseCases<SurrealUserRepository, SurrealSessionRepository, Argon2HashService>;
pub type ConcreteUserUc = UserUseCases<SurrealUserRepository, Argon2HashService>;
pub type ConcreteOrgUc = OrganizationUseCases<
    SurrealOrganizationRepository,
    SurrealUserOrganizationRepository,
    SurrealUserRepository,
>;
pub type ConcreteProjectUc =
    ProjectUseCases<SurrealProjectRepository, SurrealUserProjectRepository, SurrealUserRepository>;
pub type ConcretePipelineUc = PipelineUseCases<SurrealPipelineRepository, SurrealProjectRepository>;
pub type ConcreteJobUc = JobUseCases<SurrealJobRepository>;
pub type ConcretePermissionUc = PermissionUseCases<CasbinPermissionService>;

#[derive(Clone)]
pub struct AppState {
    pub auth_uc: Arc<ConcreteAuthUc>,
    pub user_uc: Arc<ConcreteUserUc>,
    pub org_uc: Arc<ConcreteOrgUc>,
    pub project_uc: Arc<ConcreteProjectUc>,
    pub pipeline_uc: Arc<ConcretePipelineUc>,
    pub job_uc: Arc<ConcreteJobUc>,
    pub permission_uc: Arc<ConcretePermissionUc>,
    pub permission_checker: Arc<CasbinPermissionService>,
    pub session_repo: Arc<SurrealSessionRepository>,
}

pub use routes::router;
