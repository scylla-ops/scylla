use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

// Re-export the derive macro
pub use repository_derive::Repository;

// Type alias for the diesel pool
pub type DieselPool = Pool<ConnectionManager<PgConnection>>;

// Trait générique pour repository Diesel uniquement
pub trait Repository {
    fn get_pool(&self) -> &Pool<ConnectionManager<PgConnection>>;

    fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        self.get_pool()
            .get()
            .context("Failed to get database connection (diesel)")
    }
}

// Implémentation de base
pub struct BaseRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl BaseRepository {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

impl Repository for BaseRepository {
    fn get_pool(&self) -> &Pool<ConnectionManager<PgConnection>> {
        &self.pool
    }
}
