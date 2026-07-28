//! Test scaffolding for `scylla-core`, exposed to downstream test crates via
//! the `test-utils` feature.
//!
//! Each sub-module owns one aggregate:
//! - a `*Builder` for chainable, in-memory construction with sensible defaults,
//! - a short `*(...)` free function for the zero-customisation case,
//! - a `seed_*` async helper that persists through the real repository.
//!
//! Composite scenarios (`org -> project -> pipeline -> job`) live in [`scenarios`].
//!
//! Pull everything in at once via [`prelude`]:
//! ```ignore
//! use scylla_core::test_support::prelude::*;
//! let user = UserBuilder::new("alice").inactive().build();
//! ```

pub mod authz;
pub mod job_logs;
pub mod jobs;
pub mod organizations;
pub mod pipelines;
pub mod projects;
pub mod sessions;
pub mod users;

pub mod scenarios;

pub mod prelude;
