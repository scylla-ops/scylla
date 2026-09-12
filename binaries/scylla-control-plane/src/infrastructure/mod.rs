pub mod db;
pub mod persistence;

pub use db::{close_db, init_db};

/// The in-memory and service adapters live in `scylla-core`, the Cedar adapter
/// in `scylla-auth`; re-exported so the composition root and the persistence
/// tests keep naming every adapter here.
pub use scylla_core::infrastructure::{
    Argon2HashService, CedarPermissionService, ChaChaSecretCipher, CronScheduleService,
    GitHubOAuthProvider, InMemoryAgentRegistry, InMemoryJobLogStream, LettreMailer, messaging,
    services,
};

pub use persistence::postgres::{
    PgAgentRepository, PgAppCredentialRepository, PgAppRepository, PgAppTokenRepository,
    PgAuditLog, PgAuthzEntityProvider, PgGrantRepository, PgInvitationRepository,
    PgJobLogRepository, PgJobRepository, PgOAuthIdentityRepository, PgOrganizationRepository,
    PgPipelineRepository, PgProjectRepository, PgRoleRepository, PgSecretRepository,
    PgSessionRepository, PgSignupRepository, PgTriggerDeliveryRepository, PgTriggerRepository,
    PgUserRepository,
};
