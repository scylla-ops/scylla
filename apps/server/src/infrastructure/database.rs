use include_dir::{Dir, include_dir};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb_migrations::MigrationRunner;
use tokio::sync::OnceCell;

/// Global database instance
pub static DB: OnceCell<Arc<Surreal<Any>>> = OnceCell::const_new();

/// Initialize the database connection
pub async fn init_db(url: &str, ns: &str, db: &str) -> anyhow::Result<()> {
    let client = surrealdb::engine::any::connect(url).await?;
    client.use_ns(ns).use_db(db).await?;
    DB.set(Arc::new(client))
        .map_err(|_| anyhow::anyhow!("DB already initialised"))?;
    Ok(())
}

/// Login to the database with root credentials
pub async fn login(user: &str, password: &str) -> anyhow::Result<()> {
    let db = db()?;

    db.signin(surrealdb::opt::auth::Root {
        username: user,
        password,
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to login: {:?}", e))?;

    Ok(())
}

/// Directory containing migration files
static DB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/surreal");

/// Apply database migrations
pub async fn apply_migrations(client: Arc<Surreal<Any>>) -> anyhow::Result<()> {
    MigrationRunner::new(&client)
        .load_files(&DB_DIR)
        .up()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to apply migrations: {:?}", e))?;
    Ok(())
}

/// Get the database instance
///
/// Returns an error if the database has not been initialized
pub fn db() -> anyhow::Result<Arc<Surreal<Any>>> {
    DB.get()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
        .map(|db| db.clone())
}
