//! The Community Edition's implementations of the extension points.
//!
//! This is the whole difference between editions: the core never knows which
//! one built it. Adding an extension point is adding a field to `Extensions`
//! in `scylla-extension` and a line here.

use scylla_core::application::UnlimitedQuota;
use scylla_extension::Extensions;
use std::sync::Arc;

/// The Community Edition meters nothing.
pub fn build_extensions() -> Extensions {
    Extensions {
        quota: Arc::new(UnlimitedQuota),
    }
}
