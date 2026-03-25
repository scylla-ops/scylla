use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::engine::any::connect;
use surrealdb::types::Uuid;

pub async fn init_db(tables: &[&str]) -> Surreal<Any> {
    let db = connect("memory").await.expect("Failed to connect");

    let ns = format!("test_{}", Uuid::new_v4().simple());

    db.use_ns(&ns)
        .use_db("test")
        .await
        .expect("Failed to select namespace/db");

    for table in tables {
        db.query(format!("DEFINE TABLE IF NOT EXISTS {} SCHEMALESS;", table))
            .await
            .unwrap()
            .check()
            .unwrap();
    }

    db
}
