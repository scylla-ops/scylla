use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::database::DbPool;

// Generic repository trait that provides common database functionality
pub trait Repository {
    fn get_pool(&self) -> &DbPool;

    // Helper method to get a database connection from the pool
    fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        self.get_pool()
            .get()
            .context("Failed to get database connection")
    }
}

// Base repository struct that implements the Repository trait
pub struct BaseRepository {
    pool: DbPool,
}

impl BaseRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Repository for BaseRepository {
    fn get_pool(&self) -> &DbPool {
        &self.pool
    }
}
