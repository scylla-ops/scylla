//! The extension contract between the Scylla core and an edition binary.
//!
//! An edition (the Community binary in this repository, or a private
//! Enterprise build) implements these traits and hands the core an
//! [`Extensions`] value at startup. The core calls through the traits and
//! never knows which edition it is running in.
//!
//! This crate deliberately depends on no other workspace crate: only the traits
//! and the minimal types that appear in their signatures live here, so an
//! implementation compiles outside this repository against a pinned tag
//! without pulling the domain model, the database or the gRPC stack.

pub mod quota;

pub use quota::{QuotaDecision, QuotaError, QuotaPolicy, QuotaUsage, Resource};

use std::sync::Arc;

/// The set of extension implementations an edition provides.
///
/// Built once by the binary and passed down to the services that need it.
/// Cheap to clone: every field is an `Arc`. Adding an extension point is adding
/// a field here.
#[derive(Clone)]
pub struct Extensions {
    /// Decides whether a scope may create one more resource.
    pub quota: Arc<dyn QuotaPolicy>,
}
