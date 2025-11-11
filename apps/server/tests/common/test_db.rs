//! Database setup helpers for integration tests
//!
//! This module provides utilities for creating in-memory SurrealDB instances for testing.
//! Each test should use a fresh database instance to ensure isolation.

use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// Creates a new in-memory SurrealDB instance for testing
///
/// # Example
/// ```rust
/// use common::setup_test_db;
///
/// #[tokio::test]
/// async fn test_something() {
///     let db = setup_test_db().await;
///     // Use db for testing...
/// }
/// ```
pub async fn setup_test_db() -> Arc<Surreal<Any>> {
    // Create an in-memory database using the memory protocol
    // This uses the Any engine which supports runtime database selection
    let db = surrealdb::engine::any::connect("mem://")
        .await
        .expect("Failed to create in-memory database");

    // Use a unique namespace and database for each test
    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to set namespace/database");

    // Initialize schema - define tables and constraints
    initialize_schema(&db).await;

    Arc::new(db)
}

/// Initializes the database schema with all required tables and constraints
async fn initialize_schema(db: &Surreal<Any>) {
    // Users table
    db.query(
        r#"
        DEFINE TABLE users TYPE ANY SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD created_at ON users TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
        DEFINE FIELD is_active ON users TYPE bool DEFAULT true PERMISSIONS FULL;
        DEFINE FIELD password_hash ON users TYPE string PERMISSIONS FULL;
        DEFINE FIELD updated_at ON users TYPE datetime VALUE time::now() PERMISSIONS FULL;
        DEFINE FIELD username ON users TYPE string PERMISSIONS FULL;
        DEFINE INDEX unique_username_ci ON users FIELDS username UNIQUE;
        "#,
    )
    .await
    .expect("Failed to create users table");

    // Organizations table
    db.query(
        r#"
        DEFINE TABLE organizations TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD name ON organizations TYPE string PERMISSIONS FULL;
        DEFINE FIELD description ON organizations TYPE option<string> PERMISSIONS FULL;
        DEFINE FIELD is_active ON organizations TYPE bool DEFAULT true PERMISSIONS FULL;
        DEFINE FIELD created_at ON organizations TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
        DEFINE FIELD updated_at ON organizations TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
        DEFINE INDEX unique_name ON organizations FIELDS name UNIQUE;
        "#,
    )
    .await
    .expect("Failed to create organizations table");

    // Projects table
    db.query(
        r#"
        DEFINE TABLE projects TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD name ON projects TYPE string PERMISSIONS FULL;
        DEFINE FIELD description ON projects TYPE option<string> PERMISSIONS FULL;
        DEFINE FIELD organization ON projects TYPE record<organizations> PERMISSIONS FULL;
        DEFINE FIELD is_active ON projects TYPE bool DEFAULT true PERMISSIONS FULL;
        DEFINE FIELD created_at ON projects TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
        DEFINE FIELD updated_at ON projects TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
        "#,
    )
    .await
    .expect("Failed to create projects table");

    // Pipelines table
    db.query(
        r#"
        DEFINE TABLE pipelines TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD content ON pipelines TYPE string PERMISSIONS FULL;
        DEFINE FIELD created_at ON pipelines TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
        DEFINE FIELD updated_at ON pipelines TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
        "#,
    )
    .await
    .expect("Failed to create pipelines table");

    // Jobs table
    db.query(
        r#"
        DEFINE TABLE jobs TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD content ON jobs TYPE string PERMISSIONS FULL;
        DEFINE FIELD created_at ON jobs TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
        DEFINE FIELD pipeline_id ON jobs TYPE record<pipelines> PERMISSIONS FULL;
        DEFINE FIELD status ON jobs TYPE string PERMISSIONS FULL;
        DEFINE FIELD updated_at ON jobs TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
        "#,
    )
    .await
    .expect("Failed to create jobs table");

    // User-Organization relation table
    db.query(
        r#"
        DEFINE TABLE user_organization TYPE RELATION IN users OUT organizations PERMISSIONS NONE;
        DEFINE FIELD role ON user_organization TYPE string DEFAULT "member" PERMISSIONS FULL;
        DEFINE FIELD joined_at ON user_organization TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
        DEFINE INDEX unique_user_org ON user_organization FIELDS in, out UNIQUE;
        "#,
    )
    .await
    .expect("Failed to create user_organization table");

    // User-Project relation table
    db.query(
        r#"
        DEFINE TABLE user_project TYPE RELATION IN users OUT projects PERMISSIONS NONE;
        DEFINE FIELD role ON user_project TYPE string DEFAULT "member" PERMISSIONS FULL;
        DEFINE FIELD joined_at ON user_project TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
        DEFINE INDEX unique_user_project ON user_project FIELDS in, out UNIQUE;
        "#,
    )
    .await
    .expect("Failed to create user_project table");

    // Casbin rules table (for RBAC)
    db.query(
        r#"
        DEFINE TABLE casbin_rules SCHEMAFULL;
        DEFINE FIELD ptype ON casbin_rules TYPE string;
        DEFINE FIELD v0 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v1 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v2 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v3 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v4 ON casbin_rules TYPE option<string>;
        DEFINE FIELD v5 ON casbin_rules TYPE option<string>;
        DEFINE INDEX casbin_ptype_idx ON casbin_rules COLUMNS ptype;
        "#,
    )
    .await
    .expect("Failed to create casbin_rules table");
}
