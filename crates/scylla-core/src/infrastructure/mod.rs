#[cfg(feature = "postgres")]
pub mod db;
pub mod messaging;
#[cfg(feature = "postgres")]
pub mod persistence;
pub mod services;

#[cfg(feature = "postgres")]
pub use db::{DatabaseConfig, close_db, init_db};

pub use messaging::HermesJobLogStream;

#[cfg(feature = "postgres")]
pub use persistence::postgres::PgAgentRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgJobLogRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgJobRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgOrganizationRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgPipelineRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgProjectRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgSessionRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgUserOrganizationRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgUserProjectRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgUserRepository;
#[cfg(feature = "postgres")]
pub use persistence::postgres::PgUserRoleRepository;

#[cfg(feature = "hash")]
pub use services::Argon2HashService;

#[cfg(feature = "permission")]
pub use services::CedarPermissionService;
