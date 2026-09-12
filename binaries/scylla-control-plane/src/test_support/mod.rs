//! Postgres-backed test scaffolding: the `seed_*` helpers that persist a
//! fixture through the real repository, and the composite scenarios built on
//! them.
//!
//! The in-memory builders (`*Builder`, the `org(..)`/`project(..)` shortcuts)
//! and the `PermissionService` doubles live in `scylla_core::test_support`;
//! [`prelude`] re-exports both sides so a test needs one glob import:
//! ```ignore
//! use scylla_control_plane::test_support::prelude::*;
//! let org = seed_org(&pool, "acme").await;
//! ```

pub mod scenarios;
pub mod seed;

pub mod prelude;
