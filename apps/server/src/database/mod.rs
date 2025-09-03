mod migrations;
pub mod schema;

use crate::config::core_config::DatabaseConfig;
use anyhow::{Context, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool as DPool, PooledConnection};

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
