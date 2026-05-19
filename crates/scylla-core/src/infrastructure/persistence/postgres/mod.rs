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
pub mod job_logs;
pub mod jobs;
pub mod organizations;
pub mod pipelines;
pub mod projects;
pub mod sessions;
pub mod user_organization;
pub mod user_project;
pub mod user_roles;
pub mod users;

// Flat re-exports so call sites can keep `scylla_core::infrastructure::PgUserRepository`
// without leaking the internal sub-module layout.
pub use agents::PgAgentRepository;
pub use job_logs::PgJobLogRepository;
pub use jobs::PgJobRepository;
pub use organizations::PgOrganizationRepository;
pub use pipelines::PgPipelineRepository;
pub use projects::PgProjectRepository;
pub use sessions::PgSessionRepository;
pub use user_organization::PgUserOrganizationRepository;
pub use user_project::PgUserProjectRepository;
pub use user_roles::PgUserRoleRepository;
pub use users::PgUserRepository;
