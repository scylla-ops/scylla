use anyhow::{Context, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use std::path::Path;
use tracing::info;

mod migrations;
pub mod schema;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new(config: &DatabaseConfig) -> Result<Self> {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .context("Failed to create database connection pool")?;

        info!("Database connection pool created");

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &DbPool {
        &self.pool
    }

    pub fn run_migrations(&self, migrations_dir: &Path) -> Result<()> {
        migrations::run_migrations(&self.pool, migrations_dir)
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
