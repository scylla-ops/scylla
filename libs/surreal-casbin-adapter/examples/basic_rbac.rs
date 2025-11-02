//! Basic RBAC (Role-Based Access Control) Example
//!
//! This example demonstrates how to use the SurrealDB Casbin adapter
//! for simple role-based access control.
//!
//! Run with: cargo run --example basic_rbac

use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use std::sync::Arc;
use surreal_casbin_adapter::SurrealAdapter;
use surrealdb::{Surreal, engine::local::Mem};

// Simple RBAC model
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
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;
    db.query(
        r#"
        DEFINE TABLE casbin_rules SCHEMAFULL;
        DEFINE FIELD ptype ON TABLE casbin_rules TYPE string;
        DEFINE FIELD v0 ON TABLE casbin_rules TYPE option<string>;
        DEFINE FIELD v1 ON TABLE casbin_rules TYPE option<string>;
        DEFINE FIELD v2 ON TABLE casbin_rules TYPE option<string>;
        DEFINE FIELD v3 ON TABLE casbin_rules TYPE option<string>;
        DEFINE FIELD v4 ON TABLE casbin_rules TYPE option<string>;
        DEFINE FIELD v5 ON TABLE casbin_rules TYPE option<string>;
    "#,
    )
    .await?;

    // Create enforcer
    let adapter = SurrealAdapter::new(Arc::new(db), "casbin_rules");
    let model = DefaultModel::from_str(MODEL).await?;
    let mut enforcer = Enforcer::new(model, adapter).await?;

    // Setup roles and permissions
    enforcer
        .add_policy(vec![
            "admin".to_string(),
            "data".to_string(),
            "read".to_string(),
        ])
        .await?;
    enforcer
        .add_policy(vec![
            "admin".to_string(),
            "data".to_string(),
            "write".to_string(),
        ])
        .await?;
    enforcer
        .add_policy(vec![
            "user".to_string(),
            "data".to_string(),
            "read".to_string(),
        ])
        .await?;
    enforcer
        .add_grouping_policy(vec!["alice".to_string(), "admin".to_string()])
        .await?;
    enforcer
        .add_grouping_policy(vec!["bob".to_string(), "user".to_string()])
        .await?;

    // Test permissions
    println!(
        "alice read:  {}",
        enforcer.enforce(("alice", "data", "read"))?
    );
    println!(
        "alice write: {}",
        enforcer.enforce(("alice", "data", "write"))?
    );
    println!(
        "bob read:    {}",
        enforcer.enforce(("bob", "data", "read"))?
    );
    println!(
        "bob write:   {}",
        enforcer.enforce(("bob", "data", "write"))?
    );

    Ok(())
}
