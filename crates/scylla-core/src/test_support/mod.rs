//! In-memory test scaffolding, exposed to downstream test crates via the
//! `test-utils` feature.
//!
//! Each sub-module owns one aggregate:
//! - a `*Builder` for chainable, in-memory construction with sensible defaults,
//! - a short `*(...)` free function for the zero-customisation case.
//!
//! [`authz`] holds the `PermissionService` doubles and [`quota`] a `QuotaPolicy`
//! double. Nothing here touches a
//! database: the `seed_*` helpers that persist these fixtures live next to the
//! Postgres adapters, in the crate that owns them.
//!
//! Pull everything in at once via [`prelude`]:
//! ```ignore
//! use scylla_core::test_support::prelude::*;
//! let user = UserBuilder::new("alice").is_active(false).build();
//! ```

pub mod authz;
pub mod job_logs;
pub mod jobs;
pub mod organizations;
pub mod pipelines;
pub mod projects;
pub mod quota;
pub mod sessions;
pub mod users;

pub mod prelude;
