# SurrealDB Adapter for Casbin

[![Crates.io](https://img.shields.io/crates/v/surreal-casbin-adapter.svg)](https://crates.io/crates/surreal-casbin-adapter)
[![Documentation](https://docs.rs/surreal-casbin-adapter/badge.svg)](https://docs.rs/surreal-casbin-adapter)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A SurrealDB adapter for [casbin-rs](https://github.com/casbin/casbin-rs), an authorization library that supports access control models like ACL, RBAC, ABAC for Rust projects.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
surreal-casbin-adapter = "0.1"
casbin = "2.13"
surrealdb = "2.3"
```

## Quick Start

### 1. Set up the database schema

First, create the required table in your SurrealDB database:

```sql
DEFINE TABLE casbin_rules SCHEMAFULL;

DEFINE FIELD ptype ON TABLE casbin_rules TYPE string;
DEFINE FIELD v0 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v1 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v2 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v3 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v4 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v5 ON TABLE casbin_rules TYPE option<string>;

-- Index for efficient policy lookups
DEFINE INDEX casbin_ptype_idx ON TABLE casbin_rules COLUMNS ptype;
```

### 2. Create an adapter and enforcer

```rust
use surreal_casbin_adapter::SurrealAdapter;
use casbin::prelude::*;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to SurrealDB
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

    // Sign in
    db.signin(Root {
        username: "root",
        password: "root",
    }).await?;

    // Select namespace and database
    db.use_ns("test").use_db("test").await?;

    // Create the adapter
    let adapter = SurrealAdapter::new(db, "casbin_rules");

    // Create the enforcer with your model file
    let mut enforcer = Enforcer::new("model.conf", adapter).await?;

    // Add a policy
    enforcer.add_policy(vec!["alice", "data1", "read"]).await?;

    // Check permissions
    let allowed = enforcer.enforce(("alice", "data1", "read"))?;
    println!("Alice can read data1: {}", allowed);

    Ok(())
}
```

### 3. Define your Casbin model

Create a `model.conf` file with your authorization model:

```ini
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = r.sub == p.sub && r.obj == p.obj && r.act == p.act
```

## Advanced Usage

### RBAC with Domains

For multi-tenant applications with role-based access control:

```ini
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
```

```rust
// Add role assignment
enforcer.add_grouping_policy(vec!["alice", "admin", "org1"]).await?;

// Add permission for role
enforcer.add_policy(vec!["admin", "org1", "data", "write"]).await?;

// Check permission
let allowed = enforcer.enforce(("alice", "org1", "data", "write"))?;
```

### Custom Table Name

You can specify a custom table name when creating the adapter:

```rust
let adapter = SurrealAdapter::new(db, "my_custom_casbin_table");
```

### Filtered Policy Loading

Load only specific policies to reduce memory usage:

```rust
use casbin::Filter;

let mut filter = Filter::default();
filter.p = vec!["alice"];

enforcer.load_filtered_policy(filter).await?;
```

## Examples

See [examples/](examples/) for more details.
