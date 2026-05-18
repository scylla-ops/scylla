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

#[cfg(feature = "agents")]
pub mod agents;
#[cfg(feature = "jobs")]
pub mod job_logs;
#[cfg(feature = "jobs")]
pub mod jobs;
#[cfg(feature = "organizations")]
pub mod organizations;
#[cfg(feature = "pipelines")]
pub mod pipelines;
#[cfg(feature = "projects")]
pub mod projects;
#[cfg(feature = "auth")]
pub mod sessions;
#[cfg(feature = "organizations")]
pub mod user_organization;
#[cfg(feature = "projects")]
pub mod user_project;
#[cfg(feature = "users")]
pub mod users;

// Flat re-exports so call sites can keep `scylla_core::infrastructure::PgUserRepository`
// without leaking the internal sub-module layout.
#[cfg(feature = "agents")]
pub use agents::PgAgentRepository;
#[cfg(feature = "jobs")]
pub use job_logs::PgJobLogRepository;
#[cfg(feature = "jobs")]
pub use jobs::PgJobRepository;
#[cfg(feature = "organizations")]
pub use organizations::PgOrganizationRepository;
#[cfg(feature = "pipelines")]
pub use pipelines::PgPipelineRepository;
#[cfg(feature = "projects")]
pub use projects::PgProjectRepository;
#[cfg(feature = "auth")]
pub use sessions::PgSessionRepository;
#[cfg(feature = "organizations")]
pub use user_organization::PgUserOrganizationRepository;
#[cfg(feature = "projects")]
pub use user_project::PgUserProjectRepository;
#[cfg(feature = "users")]
pub use users::PgUserRepository;
