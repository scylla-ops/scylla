pub mod agent;
pub mod audit;
pub mod auth;
pub mod caller;
pub mod job;
pub mod organization;
// The `PermissionService` trait carries no heavy deps (no cedar, no sqlx) and is
// a hard dependency of every use case, so it must compile regardless of which
// features a downstream crate (e.g. scylla-agent with default-features=false)
// enables. Only the concrete Cedar adapter stays behind the `permission` feature.
pub mod permission;
pub mod pipeline;
pub mod project;
pub mod user;
pub mod user_role;

pub use agent::{AgentRepository, AgentUseCases};
pub use audit::{AuditDecision, AuditEntry, AuditLog, NoopAuditLog};
pub use auth::{AuthUseCases, HashService, SessionRepository};
pub use caller::{CallerContext, ServiceIdentity};
pub use job::{
    JobLogLiveStream, JobLogRepository, JobLogStreamPort, JobLogStreamUseCase, JobLogUseCases,
    JobRepository, JobUseCases,
};
pub use organization::{OrganizationRepository, OrganizationUseCases, UserOrganizationRepository};
pub use permission::{
    AuthzEntityProvider, Grant, GrantRepository, GrantScope, GrantUseCases, PermissionService,
    PolicyControl, PolicyDefinition, PolicyRepository, PolicyUseCases, PrincipalAuthz,
    ResourceAncestors,
};
pub use pipeline::{PipelineRepository, PipelineUseCases};
pub use project::{ProjectRepository, ProjectUseCases, UserProjectRepository};
pub use user::{UserRepository, UserUseCases};
pub use user_role::{UserRoleRepository, UserRoleUseCases};
