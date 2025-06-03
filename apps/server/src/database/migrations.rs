use anyhow::{Context, Result, anyhow};
use diesel::Connection;
use diesel::RunQueryDsl;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use std::fs;
use std::path::Path;
use tracing::info;

pub fn run_migrations(
    pool: &Pool<ConnectionManager<PgConnection>>,
    migrations_dir: &Path,
) -> Result<()> {
    info!(
        "Running database migrations from {}",
        migrations_dir.display()
    );

    // Get a connection from the pool
    let mut conn = pool.get().context("Failed to get database connection")?;

    // Read migration files from the directory
    let mut migration_files = fs::read_dir(migrations_dir)
        .context("Failed to read migrations directory")?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
        .collect::<Vec<_>>();

    // Sort migration files by name
    migration_files.sort();

    // Apply each migration in a transaction
    for migration_path in migration_files {
        let migration_name = migration_path
            .file_name()
            .context("Failed to get migration file name")?
            .to_string_lossy()
            .into_owned();

        info!("Applying migration: {}", migration_name);

        // Read migration file
        let migration_sql = fs::read_to_string(&migration_path).with_context(|| {
            format!(
                "Failed to read migration file: {}",
                migration_path.display()
            )
        })?;

        // Run the migration in a transaction
        conn.transaction(|conn| {
            sql_query(&migration_sql)
                .execute(conn)
                .map_err(|e| anyhow!("Failed to execute migration: {}", e))
        })
        .map_err(|e| anyhow!("Migration failed: {}", e))?;

        info!("Migration applied successfully: {}", migration_name);
    }

    info!("Database migrations completed successfully");
    Ok(())
}
