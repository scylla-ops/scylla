use clap::Parser;
use scylla_core::infrastructure::DatabaseConfig;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "scylla-recorder",
    about = "Scylla event recorder — persists job status and logs from the broker to the database"
)]
pub struct RecorderConfig {
    /// Hermes broker gRPC URL
    #[arg(long, default_value = "http://127.0.0.1:50052")]
    pub broker_url: String,

    /// SurrealDB connection URL
    #[arg(long, default_value = "ws://127.0.0.1:8000")]
    pub db_url: String,

    /// SurrealDB namespace
    #[arg(long, default_value = "scylla")]
    pub db_namespace: String,

    /// SurrealDB database
    #[arg(long, default_value = "core")]
    pub db_database: String,

    /// SurrealDB username
    #[arg(long, default_value = "root")]
    pub db_username: String,

    /// SurrealDB password
    #[arg(long, default_value = "root")]
    pub db_password: String,
}

impl RecorderConfig {
    #[must_use]
    pub fn database_config(&self) -> DatabaseConfig {
        DatabaseConfig {
            url: self.db_url.clone(),
            namespace: self.db_namespace.clone(),
            database: self.db_database.clone(),
            username: self.db_username.clone(),
            password: self.db_password.clone(),
        }
    }
}
