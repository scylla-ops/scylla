#[cfg(feature = "postgres")]
pub mod db;
pub mod messaging;
#[cfg(feature = "postgres")]
pub mod persistence;
pub mod services;

#[cfg(feature = "postgres")]
pub use db::{DatabaseConfig, close_db, init_db};

#[cfg(feature = "jobs")]
pub use messaging::HermesJobLogStream;

#[cfg(all(feature = "postgres", feature = "agents"))]
pub use persistence::postgres::PgAgentRepository;
#[cfg(all(feature = "postgres", feature = "jobs"))]
pub use persistence::postgres::PgJobLogRepository;
#[cfg(all(feature = "postgres", feature = "jobs"))]
pub use persistence::postgres::PgJobRepository;
#[cfg(all(feature = "postgres", feature = "organizations"))]
pub use persistence::postgres::PgOrganizationRepository;
#[cfg(all(feature = "postgres", feature = "pipelines"))]
pub use persistence::postgres::PgPipelineRepository;
#[cfg(all(feature = "postgres", feature = "projects"))]
pub use persistence::postgres::PgProjectRepository;
#[cfg(all(feature = "postgres", feature = "auth"))]
pub use persistence::postgres::PgSessionRepository;
#[cfg(all(feature = "postgres", feature = "organizations"))]
pub use persistence::postgres::PgUserOrganizationRepository;
#[cfg(all(feature = "postgres", feature = "projects"))]
pub use persistence::postgres::PgUserProjectRepository;
#[cfg(all(feature = "postgres", feature = "users"))]
pub use persistence::postgres::PgUserRepository;

#[cfg(feature = "hash")]
pub use services::Argon2HashService;

#[cfg(feature = "permission")]
pub use services::CasbinPermissionService;
