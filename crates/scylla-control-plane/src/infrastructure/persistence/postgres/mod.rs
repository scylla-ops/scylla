//! `PostgreSQL` persistence adapters, grouped by aggregate.
//!
//! Each sub-module owns one aggregate and contains:
//! - `repository.rs`: the trait impl + the SQL via `pub mod queries`.
//! - `tests.rs`: integration tests via `#[sqlx::test]` against a real Postgres.
//!
//! The query helpers take any `sqlx::PgExecutor`, so the same SQL is reused
//! both from pool-backed repos and from ad-hoc transactions
//! (`pool.begin().await`).

mod error;

pub mod agents;
pub mod app_secrets;
pub mod app_tokens;
pub mod apps;
pub mod audit;
pub mod authz;
pub mod grants;
pub mod invitations;
pub mod job_logs;
pub mod jobs;
pub mod oauth_identities;
pub mod organizations;
pub mod pipelines;
pub mod project_secrets;
pub mod projects;
pub mod roles;
pub mod sessions;
pub mod signup;
pub mod trigger_deliveries;
pub mod triggers;
pub mod users;

// Flat re-exports so call sites can keep `crate::infrastructure::PgUserRepository`
// without leaking the internal sub-module layout.
pub use agents::PgAgentRepository;
pub use app_secrets::PgAppCredentialRepository;
pub use app_tokens::PgAppTokenRepository;
pub use apps::PgAppRepository;
pub use audit::PgAuditLog;
pub use authz::PgAuthzEntityProvider;
pub use grants::PgGrantRepository;
pub use invitations::PgInvitationRepository;
pub use job_logs::PgJobLogRepository;
pub use jobs::PgJobRepository;
pub use oauth_identities::PgOAuthIdentityRepository;
pub use organizations::PgOrganizationRepository;
pub use pipelines::PgPipelineRepository;
pub use project_secrets::PgSecretRepository;
pub use projects::PgProjectRepository;
pub use roles::PgRoleRepository;
pub use sessions::PgSessionRepository;
pub use signup::PgSignupRepository;
pub use trigger_deliveries::PgTriggerDeliveryRepository;
pub use triggers::PgTriggerRepository;
pub use users::PgUserRepository;
