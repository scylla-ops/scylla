pub mod macros;
mod migrations;
pub mod schema;

use anyhow::{Context, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool as DPool, PooledConnection};
use tower_sessions_sqlx_store::sqlx::{PgPool as SPool, postgres::PgPoolOptions};

// Type alias for the diesel pool
pub type DieselPool = DPool<ConnectionManager<PgConnection>>;
pub type DieselConnection = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct DieselDatabase {
    pub pool: DieselPool,
}

impl DieselDatabase {
    pub fn new(config: &DatabaseConfig) -> Result<Self> {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = DPool::builder()
            .build(manager)
            .context("Failed to create diesel database connection pool")?;
        Ok(Self { pool })
    }

    pub fn run_migrations(&self) -> Result<()> {
        migrations::run_migrations(&self.pool)
    }
}

#[derive(Clone)]
pub struct SqlxDatabase {
    pub pool: SPool,
}

impl SqlxDatabase {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .context("Failed to create sqlx database connection pool")?;
        Ok(Self { pool })
    }
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn new(
        host: String,
        port: u16,
        username: String,
        password: String,
        database: String,
    ) -> Self {
        Self {
            host,
            port,
            username,
            password,
            database,
        }
    }
}
