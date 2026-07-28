#[cfg(feature = "postgres")]
pub mod db;
pub mod messaging;
#[cfg(feature = "postgres")]
pub mod persistence;
pub mod services;

#[cfg(feature = "postgres")]
pub use db::{DatabaseConfig, close_db, init_db};

pub use messaging::{InMemoryAgentRegistry, InMemoryJobLogStream};

pub use services::{ChaChaSecretCipher, CronScheduleService, GitHubOAuthProvider, LettreMailer};

#[cfg(feature = "hash")]
pub use services::Argon2HashService;

#[cfg(feature = "permission")]
pub use services::CedarPermissionService;

#[cfg(feature = "postgres")]
pub use persistence::postgres::{
    PgAgentRepository, PgAppCredentialRepository, PgAppRepository, PgAppTokenRepository, PgAuditLog,
    PgAuthzEntityProvider, PgGrantRepository, PgInvitationRepository, PgJobLogRepository,
    PgJobRepository, PgOAuthIdentityRepository, PgOrganizationRepository, PgPipelineRepository,
    PgProjectRepository, PgRoleRepository, PgSecretRepository, PgSessionRepository,
    PgSignupRepository, PgTriggerDeliveryRepository, PgTriggerRepository, PgUserRepository,
};
