//! The Postgres adapters: one `Pg*` repository per aggregate implementing the
//! ports declared in `scylla-core` and `scylla-auth`, the connection pool, and
//! the embedded migrations.
//!
//! This crate sits above `scylla-core` (it implements core's traits) and below
//! the composition root that hands the repositories to the use cases. It is
//! the only crate that runs queries; everything above it only passes the pool
//! around.

/// The domain model, re-exported from the [`scylla_domain`] kernel so that
/// `crate::domain::...` paths keep naming it from anywhere in this crate.
#[doc(no_inline)]
pub use scylla_domain::domain;

pub mod pool;
pub mod postgres;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_support;

pub use pool::{close_db, init_db};

pub use postgres::{
    PgAgentRepository, PgAppCredentialRepository, PgAppRepository, PgAppTokenRepository,
    PgAuditLog, PgAuthzEntityProvider, PgGrantRepository, PgInvitationRepository,
    PgJobLogRepository, PgJobRepository, PgOAuthIdentityRepository, PgOrganizationRepository,
    PgPipelineRepository, PgProjectRepository, PgRoleRepository, PgSecretRepository,
    PgSessionRepository, PgSignupRepository, PgTriggerDeliveryRepository, PgTriggerRepository,
    PgUserRepository,
};
