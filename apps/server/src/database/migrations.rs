use anyhow::{Context, Result, anyhow};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn run_migrations(pool: &Pool<ConnectionManager<PgConnection>>) -> Result<()> {
    // Get a connection from the pool
    let mut conn = pool.get().context("Failed to get database connection")?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow!(e.to_string()))
        .context("Failed to run pending migrations")?;

    Ok(())
}
