pub mod agent;
pub mod app;
pub mod audit;
pub mod auth;
pub mod caller;
#[cfg(feature = "invitations")]
pub mod invitation;
pub mod job;
pub mod mail;
#[cfg(feature = "oauth-github")]
pub mod oauth;
pub mod organization;
// The `PermissionService` trait carries no heavy deps (no cedar, no sqlx) and is
// a hard dependency of every use case, so it must compile regardless of which
// features a downstream crate (e.g. scylla-agent with default-features=false)
// enables. Only the concrete Cedar adapter stays behind the `permission` feature.
pub mod permission;
pub mod pipeline;
pub mod project;
pub mod quota;
pub mod signup;
pub mod user;
pub mod user_role;

pub use agent::{AgentRepository, AgentUseCases};
pub use app::{AppRepository, AppUseCases, CreatedApp};
pub use audit::{AuditDecision, AuditEntry, AuditLog, NoopAuditLog};
pub use auth::{AuthUseCases, HashService, SessionRepository};
pub use caller::{CallerContext, ServiceIdentity};
pub use job::{
    JobLogLiveStream, JobLogRepository, JobLogStreamPort, JobLogStreamUseCase, JobLogUseCases,
    JobRepository, JobUseCases,
};
#[cfg(feature = "invitations")]
pub use invitation::{AcceptOutcome, InvitationRepository, InvitationUseCases};
pub use mail::{Mailer, NoopMailer};
#[cfg(feature = "oauth-github")]
pub use oauth::{
    OAuthIdentityRepository, OAuthOutcome, OAuthProvider, OAuthUseCases, OAuthUserInfo,
};
pub use organization::{OrganizationRepository, OrganizationUseCases, UserOrganizationRepository};
pub use permission::{
    AuthzEntityProvider, Grant, GrantPrincipal, GrantRepository, GrantScope, GrantUseCases,
    PermissionService, PolicyControl, PolicyDefinition, PolicyRepository, PolicyUseCases,
    PrincipalAuthz, ResourceAncestors,
};
pub use pipeline::{PipelineRepository, PipelineUseCases};
pub use project::{ProjectRepository, ProjectUseCases, UserProjectRepository};
pub use quota::Quotas;
pub use signup::{SignupOutcome, SignupRepository, SignupUseCases};
pub use user::{UserRepository, UserUseCases};
pub use user_role::{UserRoleRepository, UserRoleUseCases};
