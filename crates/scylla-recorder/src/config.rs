use clap::Parser;
use scylla_core::infrastructure::DatabaseConfig;
use std::time::Duration;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "scylla-recorder",
    about = "Scylla event recorder — persists job status and logs from the broker to the database"
)]
pub struct RecorderConfig {
    /// Hermes broker gRPC URL
    #[arg(long, default_value = "http://127.0.0.1:50052")]
    pub broker_url: String,

    /// PostgreSQL connection URL (e.g. `postgres://user:pass@host:5432/dbname`).
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://scylla:scylla@localhost:5432/scylla"
    )]
    pub db_url: String,

    /// Maximum number of pooled connections.
    #[arg(long, default_value_t = 8)]
    pub db_max_connections: u32,

    /// Apply migrations on startup. Set to false if a sibling service manages schema.
    #[arg(long, default_value_t = false)]
    pub db_run_migrations: bool,
}

impl RecorderConfig {
    #[must_use]
    pub fn database_config(&self) -> DatabaseConfig {
        DatabaseConfig {
            url: self.db_url.clone(),
            max_connections: self.db_max_connections,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(30),
            run_migrations: self.db_run_migrations,
        }
    }
}
