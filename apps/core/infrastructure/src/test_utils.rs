use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::engine::any::connect;
// use surrealdb::opt::auth::Root;
use surrealdb::types::Uuid;

pub async fn init_db(tables: &[&str]) -> Surreal<Any> {
    // let db = connect("ws://localhost:8000")
    //     .await
    //     .expect("Failed to connect");

    let db = connect("memory").await.expect("Failed to connect");

    // db.signin(Root {
    //     username: "root".into(),
    //     password: "secret".into(),
    // })
    // .await
    // .expect("Failed to sign in");

    let ns = format!("test_{}", Uuid::new_v4().simple());
    let db_name = "test";

    db.use_ns(&ns)
        .use_db(db_name)
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
