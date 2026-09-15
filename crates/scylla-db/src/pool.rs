use crate::domain::errors::{DomainError, DomainResult};
use scylla_core::config::DatabaseConfig;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

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

pub async fn close_db(pool: &PgPool) {
    pool.close().await;
    tracing::debug!("PostgreSQL pool closed");
}
