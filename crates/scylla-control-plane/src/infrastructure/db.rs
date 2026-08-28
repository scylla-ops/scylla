use crate::domain::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// Accepted as a human-friendly duration: `"30s"`, `"500ms"`, `"1m"`.
    #[serde(default = "default_acquire_timeout", with = "humantime_serde")]
    pub acquire_timeout: Duration,
    #[serde(default)]
    pub run_migrations: bool,
}

const fn default_max_connections() -> u32 {
    16
}

const fn default_min_connections() -> u32 {
    1
}

const fn default_acquire_timeout() -> Duration {
    Duration::from_secs(30)
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://scylla:scylla@localhost:5432/scylla".to_string(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            acquire_timeout: default_acquire_timeout(),
            run_migrations: true,
        }
    }
}

/// Connect to `PostgreSQL` using a connection pool and, if `run_migrations` is
/// true, apply pending migrations from the bundled `migrations/` directory at
/// the workspace root.
pub async fn init_db(config: &DatabaseConfig) -> DomainResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .connect(&config.url)
        .await
        .map_err(|e| {
            DomainError::infrastructure(format!(
                "Failed to connect to database at {}: {e}",
                config.url
            ))
        })?;

    if config.run_migrations {
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("Failed to apply database migrations: {e}"))
            })?;
        tracing::info!("Database migrations applied successfully");
    }

    Ok(pool)
}

/// Gracefully close the pool. Waits for in-flight queries to complete.
pub async fn close_db(pool: &PgPool) {
    pool.close().await;
    tracing::debug!("PostgreSQL pool closed");
}
