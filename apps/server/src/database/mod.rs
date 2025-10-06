use include_dir::{Dir, include_dir};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb_migrations::MigrationRunner;
use tokio::sync::OnceCell;

pub static DB: OnceCell<Arc<Surreal<Any>>> = OnceCell::const_new();

pub async fn init_db(url: &str, ns: &str, db: &str) -> anyhow::Result<()> {
    let client = surrealdb::engine::any::connect(url).await?;
    client.use_ns(ns).use_db(db).await?;
    DB.set(Arc::new(client))
        .map_err(|_| anyhow::anyhow!("DB already initialised"))?;
    Ok(())
}

pub async fn login(user: &str, password: &str) -> anyhow::Result<()> {
    let db = db();

    db.signin(surrealdb::opt::auth::Root {
        username: user,
        password,
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to login: {:?}", e))?;

    Ok(())
}

static DB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/surreal");

pub async fn apply_migrations(client: Arc<Surreal<Any>>) {
    MigrationRunner::new(&client)
        .load_files(&DB_DIR)
        .up()
        .await
        .expect("Failed to apply migrations");
}

pub fn db() -> Arc<Surreal<Any>> {
    DB.get().expect("DB not initialised").clone()
}
