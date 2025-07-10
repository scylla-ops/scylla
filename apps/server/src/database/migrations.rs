use crate::database::DieselPool;
use anyhow::{Context, Result, anyhow};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(pool: &DieselPool) -> Result<()> {
    // Get a connection from the pool
    let mut conn = pool.get().context("Failed to get database connection")?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow!(e.to_string()))
        .context("Failed to run pending migrations")?;

    Ok(())
}
