pub mod app;
pub mod audit;
pub mod auth;
pub mod bootstrap;
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
pub mod agent;
pub mod authz;
pub mod pipeline;
pub mod project;
pub mod quota;
pub mod signup;
pub mod user;

pub use agent::{
    AgentDispatch, AgentRepository, AgentStats, AgentUseCases, AgentView, CreatedAgent,
    DispatchOutcome, DispatchUseCases, JobDispatch, PendingJobScheduler,
};
pub use app::{
    AppCredentialRepository, AppRepository, AppTokenOutcome, AppTokenRepository, AppTokenUseCases,
    AppUseCases, CreatedApp, CreatedAppSecret,
};
pub use audit::{AuditDecision, AuditEntry, AuditLog, NoopAuditLog};
pub use auth::{AuthUseCases, HashService, SessionRepository};
pub use authz::{
    AuthzEntityProvider, EffectiveScope, FULL_CONTROL, Grant, GrantRepository, GrantTarget,
    GrantUseCases, GrantableRole, PermissionService, PolicyControl, PolicyDefinition,
    PolicyRepository, PolicyUseCases, Principal, PrincipalAuthz, ResourceAncestors, Role, RoleKind,
    RoleRepository, RoleUseCases, Scope, ScopeKind, grantable_roles, resource_home_scope,
    validate_role_in_db,
};
pub use bootstrap::BootstrapUseCases;
pub use caller::{CallerContext, ServiceIdentity};
#[cfg(feature = "invitations")]
pub use invitation::{AcceptOutcome, InvitationRepository, InvitationUseCases};
pub use job::{
    JobEvent, JobLogLiveStream, JobLogRepository, JobLogStreamPort, JobLogStreamUseCase,
    JobLogUseCases, JobRepository, JobUseCases,
};
pub use mail::{Mailer, NoopMailer};
#[cfg(feature = "oauth-github")]
pub use oauth::{
    OAuthIdentityRepository, OAuthOutcome, OAuthProvider, OAuthUseCases, OAuthUserInfo,
};
pub use organization::{OrganizationRepository, OrganizationUseCases, UserOrganizationRepository};
pub use pipeline::{PipelineRepository, PipelineUseCases};
pub use project::{ProjectRepository, ProjectUseCases, UserProjectRepository};
pub use quota::Quotas;
pub use signup::{SignupOutcome, SignupRepository, SignupUseCases};
pub use user::{UserRepository, UserUseCases};
