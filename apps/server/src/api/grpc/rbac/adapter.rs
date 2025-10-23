//! Re-export of the generic SurrealDB Casbin adapter
//!
//! This module provides a wrapper around the generic surreal-casbin-adapter
//! to integrate it with Scylla's database connection.

use crate::api::grpc::tables;
use crate::database::db;

// Re-export the adapter type
pub use surreal_casbin_adapter::SurrealAdapter as GenericSurrealAdapter;

/// Type alias for the SurrealAdapter using Scylla's database connection
pub type SurrealAdapter = GenericSurrealAdapter<surrealdb::engine::any::Any>;

/// Creates a new SurrealAdapter using Scylla's database connection
pub fn new_adapter() -> SurrealAdapter {
    GenericSurrealAdapter::new(db(), tables::casbin_rules::TABLE)
}
