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

#[cfg(feature = "agents")]
pub mod agents;
#[cfg(feature = "jobs")]
pub mod job_logs;
#[cfg(feature = "jobs")]
pub mod jobs;
#[cfg(feature = "organizations")]
pub mod organizations;
#[cfg(feature = "pipelines")]
pub mod pipelines;
#[cfg(feature = "projects")]
pub mod projects;
#[cfg(feature = "auth")]
pub mod sessions;
#[cfg(feature = "users")]
pub mod users;

#[cfg(feature = "pipelines")]
pub mod scenarios;

pub mod prelude;
