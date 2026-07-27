#[cfg(feature = "postgres")]
pub mod db;
pub mod messaging;
#[cfg(feature = "postgres")]
pub mod persistence;
pub mod services;

#[cfg(feature = "postgres")]
pub use db::{DatabaseConfig, close_db, init_db};

pub use messaging::{InMemoryAgentRegistry, InMemoryJobLogStream};

pub use services::ChaChaSecretCipher;
pub use services::CronScheduleService;

#[cfg(feature = "postgres")]
pub use persistence::postgres::PgAgentRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgAppCredentialRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgAppRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgAppTokenRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgAuditLog;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgAuthzEntityProvider;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgGrantRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgJobLogRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgJobRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgOrganizationRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgPipelineRepository;
#[cfg(feature = "postgres")]
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgProjectRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgRoleRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgSecretRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgSessionRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgSignupRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgTriggerDeliveryRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgTriggerRepository;
#[cfg(feature = "postgres")]
#[cfg(feature = "postgres")]
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgUserRepository;
#[cfg(feature = "postgres")]
#[cfg(feature = "hash")]
pub use services::Argon2HashService;

#[cfg(feature = "permission")]
pub use services::CedarPermissionService;

pub use services::LettreMailer;

#[cfg(feature = "postgres")]
pub use persistence::postgres::PgInvitationRepository;

#[cfg(feature = "postgres")]
pub use persistence::postgres::PgOAuthIdentityRepository;
pub use services::GitHubOAuthProvider;
