use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use tokio::sync::OnceCell;

pub static DB: OnceCell<Arc<Surreal<Client>>> = OnceCell::const_new();

pub async fn init_db(_url: &str, ns: &str, db: &str) -> anyhow::Result<()> {
    let client = Surreal::new::<Ws>("127.0.0.1:8000").await?;
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

pub fn db() -> Arc<Surreal<Client>> {
    DB.get().expect("DB not initialised").clone()
}
