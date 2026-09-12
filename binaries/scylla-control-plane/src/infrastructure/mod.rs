//! Every adapter the composition root wires, re-exported from the crates that
//! own them: the Postgres repositories and pool from `scylla-db`, the in-memory
//! and service adapters from `scylla-core`, the Cedar adapter from
//! `scylla-auth`.

pub use scylla_db::{close_db, init_db};

pub use scylla_core::infrastructure::{
    Argon2HashService, CedarPermissionService, ChaChaSecretCipher, CronScheduleService,
    GitHubOAuthProvider, InMemoryAgentRegistry, InMemoryJobLogStream, LettreMailer, messaging,
    services,
};

pub use scylla_db::{
    PgAgentRepository, PgAppCredentialRepository, PgAppRepository, PgAppTokenRepository,
    PgAuditLog, PgAuthzEntityProvider, PgGrantRepository, PgInvitationRepository,
    PgJobLogRepository, PgJobRepository, PgOAuthIdentityRepository, PgOrganizationRepository,
    PgPipelineRepository, PgProjectRepository, PgRoleRepository, PgSecretRepository,
    PgSessionRepository, PgSignupRepository, PgTriggerDeliveryRepository, PgTriggerRepository,
    PgUserRepository,
};
