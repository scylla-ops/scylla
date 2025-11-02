//! Multi-Tenant RBAC with Domains Example
//!
//! This example demonstrates how to use the SurrealDB Casbin adapter
//! for multi-tenant applications using domains (organizations/projects).
//!
//! Run with: cargo run --example multi_tenant

use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi, RbacApi};
use std::sync::Arc;
use surreal_casbin_adapter::SurrealAdapter;
use surrealdb::{Surreal, engine::local::Mem};

// RBAC model with domains
const MODEL: &str = r#"
[request_definition]
r = sub, dom, obj, act

[policy_definition]
p = sub, dom, obj, act

[role_definition]
g = _, _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub, r.dom) && r.dom == p.dom && r.obj == p.obj && r.act == p.act
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

    // Setup acme-corp organization
    enforcer
        .add_policy(vec![
            "owner".to_string(),
            "acme-corp".to_string(),
            "projects".to_string(),
            "write".to_string(),
        ])
        .await?;
    enforcer
        .add_policy(vec![
            "member".to_string(),
            "acme-corp".to_string(),
            "projects".to_string(),
            "read".to_string(),
        ])
        .await?;
    enforcer
        .add_grouping_policy(vec![
            "alice".to_string(),
            "owner".to_string(),
            "acme-corp".to_string(),
        ])
        .await?;
    enforcer
        .add_grouping_policy(vec![
            "bob".to_string(),
            "member".to_string(),
            "acme-corp".to_string(),
        ])
        .await?;

    // Setup tech-startup organization
    enforcer
        .add_policy(vec![
            "owner".to_string(),
            "tech-startup".to_string(),
            "projects".to_string(),
            "write".to_string(),
        ])
        .await?;
    enforcer
        .add_policy(vec![
            "member".to_string(),
            "tech-startup".to_string(),
            "projects".to_string(),
            "read".to_string(),
        ])
        .await?;
    enforcer
        .add_grouping_policy(vec![
            "bob".to_string(),
            "owner".to_string(),
            "tech-startup".to_string(),
        ])
        .await?;
    enforcer
        .add_grouping_policy(vec![
            "charlie".to_string(),
            "member".to_string(),
            "tech-startup".to_string(),
        ])
        .await?;

    // Test permissions
    println!("acme-corp:");
    println!(
        "  alice write: {}",
        enforcer.enforce(("alice", "acme-corp", "projects", "write"))?
    );
    println!(
        "  bob write:   {}",
        enforcer.enforce(("bob", "acme-corp", "projects", "write"))?
    );
    println!(
        "  bob read:    {}",
        enforcer.enforce(("bob", "acme-corp", "projects", "read"))?
    );

    println!("\ntech-startup:");
    println!(
        "  bob write:     {}",
        enforcer.enforce(("bob", "tech-startup", "projects", "write"))?
    );
    println!(
        "  charlie write: {}",
        enforcer.enforce(("charlie", "tech-startup", "projects", "write"))?
    );
    println!(
        "  alice write:   {} (isolated)",
        enforcer.enforce(("alice", "tech-startup", "projects", "write"))?
    );

    println!(
        "\nbob roles: acme-corp={:?}, tech-startup={:?}",
        enforcer.get_roles_for_user("bob", Some("acme-corp")),
        enforcer.get_roles_for_user("bob", Some("tech-startup"))
    );

    Ok(())
}
