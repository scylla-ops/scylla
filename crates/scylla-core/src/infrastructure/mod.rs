#[cfg(feature = "surrealdb")]
pub mod db;
#[cfg(feature = "surrealdb")]
pub mod persistence;
pub mod services;

#[cfg(test)]
pub(crate) mod test_utils;

#[cfg(feature = "surrealdb")]
pub use db::{Db, DatabaseConfig, close_db, init_db};

#[cfg(all(feature = "surrealdb", feature = "jobs"))]
pub use persistence::surrealdb::SurrealJobLogRepository;
#[cfg(all(feature = "surrealdb", feature = "jobs"))]
pub use persistence::surrealdb::SurrealJobRepository;
#[cfg(all(feature = "surrealdb", feature = "organizations"))]
pub use persistence::surrealdb::SurrealOrganizationRepository;
#[cfg(all(feature = "surrealdb", feature = "pipelines"))]
pub use persistence::surrealdb::SurrealPipelineRepository;
#[cfg(all(feature = "surrealdb", feature = "projects"))]
pub use persistence::surrealdb::SurrealProjectRepository;
#[cfg(all(feature = "surrealdb", feature = "auth"))]
pub use persistence::surrealdb::SurrealSessionRepository;
#[cfg(all(feature = "surrealdb", feature = "organizations"))]
pub use persistence::surrealdb::SurrealUserOrganizationRepository;
#[cfg(all(feature = "surrealdb", feature = "projects"))]
pub use persistence::surrealdb::SurrealUserProjectRepository;
#[cfg(all(feature = "surrealdb", feature = "users"))]
pub use persistence::surrealdb::SurrealUserRepository;

#[cfg(feature = "hash")]
pub use services::Argon2HashService;

#[cfg(feature = "permission")]
pub use services::CasbinPermissionService;
