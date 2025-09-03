use anyhow::{Context, Result};

use crate::database::{DieselConnection, DieselPool};
// Re-export the derive macro
pub use repository_derive::DieselRepository;

// Trait générique pour repository Diesel uniquement
pub trait Repository {
    fn get_pool(&self) -> &DieselPool;

    fn get_connection(&self) -> Result<DieselConnection> {
        self.get_pool()
            .get()
            .context("Failed to get database connection (diesel)")
    }
}

// Implémentation de base
pub struct BaseRepository {
    pool: DieselPool,
}

impl BaseRepository {
    pub fn new(pool: DieselPool) -> Self {
        Self { pool }
    }
}

impl Repository for BaseRepository {
    fn get_pool(&self) -> &DieselPool {
        &self.pool
    }
}
