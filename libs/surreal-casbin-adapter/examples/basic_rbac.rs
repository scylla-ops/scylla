//! Basic RBAC (Role-Based Access Control) Example
//!
//! This example demonstrates how to use the SurrealDB Casbin adapter
//! for simple role-based access control with both g and g2.
//!
//! Run with: cargo run --example basic_rbac

use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use surreal_casbin_adapter::SurrealAdapter;
use surrealdb::engine::any::connect;
use surrealdb::opt::auth::Root;

// RBAC model avec g et g2
const MODEL: &str = r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize SurrealDB
    let db = connect("ws://localhost:8000").await?;
    db.signin(Root {
        username: "root".to_string(),
        password: "secret".to_string(),
    })
    .await?;
    db.use_ns("test").use_db("test").await?;
    db.query("DEFINE TABLE IF NOT EXISTS $table SCHEMALESS;")
        .bind(("table", surreal_casbin_adapter::TABLE))
        .await?;

    // Create enforcer
    let adapter = SurrealAdapter::new(db);
    let model = DefaultModel::from_str(MODEL).await?;
    let mut enforcer = Enforcer::new(model, adapter).await?;

    // ─── Permissions ────────────────────────────────────────────────────────
    enforcer
        .add_policy(vec![
            "reader".to_string(),
            "data".to_string(),
            "read".to_string(),
        ])
        .await?;

    enforcer
        .add_named_grouping_policies("g", vec![vec!["alice".to_string(), "reader".to_string()]])
        .await?;

    Ok(())
}
