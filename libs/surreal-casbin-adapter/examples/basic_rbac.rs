//! Basic RBAC (Role-Based Access Control) Example
//!
//! This example demonstrates how to use the SurrealDB Casbin adapter
//! for simple role-based access control with both g and g2.
//!
//! Run with: cargo run --example basic_rbac

use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi, RbacApi};
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
g2 = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = (g(r.sub, p.sub) || g2(r.sub, p.sub)) && r.obj == p.obj && r.act == p.act
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

    // ─── Rôles via g (alice) ─────────────────────────────────────────────────
    enforcer
        .add_named_grouping_policy("g", vec!["alice".to_string(), "admin".to_string()])
        .await?;

    // ─── Rôles via g2 (charlie) ──────────────────────────────────────────────
    enforcer
        .add_named_grouping_policy("g2", vec!["charlie".to_string(), "admin".to_string()])
        .await?;

    // ─── Rôles via g (bob) ───────────────────────────────────────────────────
    enforcer
        .add_grouping_policy(vec!["bob".to_string(), "user".to_string()])
        .await?;

    // ═══════════════════════════════════════════════════════════════════════════
    println!("─── Test enforce ────────────────────────────────────");

    // alice est dans g → admin
    println!(
        "alice  read  (g)  : {}",
        enforcer.enforce(("alice", "data", "read"))?
    ); // true
    println!(
        "alice  write (g)  : {}",
        enforcer.enforce(("alice", "data", "write"))?
    ); // true

    // charlie est dans g2 → admin, le matcher utilise g2 aussi
    println!(
        "charlie read  (g2): {}",
        enforcer.enforce(("charlie", "data", "read"))?
    ); // true
    println!(
        "charlie write (g2): {}",
        enforcer.enforce(("charlie", "data", "write"))?
    ); // true

    // bob est dans g → user (read only)
    println!(
        "bob    read  (g)  : {}",
        enforcer.enforce(("bob", "data", "read"))?
    ); // true
    println!(
        "bob    write (g)  : {}",
        enforcer.enforce(("bob", "data", "write"))?
    ); // false

    // ═══════════════════════════════════════════════════════════════════════════
    println!("\n─── Rôles ───────────────────────────────────────────");

    println!(
        "roles alice   (g)  : {:?}",
        enforcer.get_roles_for_user("alice", None)
    ); // ["admin"]
    println!(
        "roles charlie (g2) : {:?}",
        enforcer.get_roles_for_user("charlie", None)
    ); // [] ← g2 non résolu par get_roles_for_user
    println!(
        "roles bob     (g)  : {:?}",
        enforcer.get_roles_for_user("bob", None)
    ); // ["user"]

    // ═══════════════════════════════════════════════════════════════════════════
    println!("\n─── Permissions implicites ──────────────────────────");

    println!(
        "implicit alice   : {:?}",
        enforcer.get_implicit_permissions_for_user("alice", None)
    );
    println!(
        "implicit charlie : {:?}",
        enforcer.get_implicit_permissions_for_user("charlie", None)
    );
    println!(
        "implicit bob     : {:?}",
        enforcer.get_implicit_permissions_for_user("bob", None)
    );

    // Tout ce qui est dans g
    println!("tout g  : {:?}", enforcer.get_named_grouping_policy("g"));

    // Tout ce qui est dans g2
    println!("tout g2 : {:?}", enforcer.get_named_grouping_policy("g2"));

    Ok(())
}
