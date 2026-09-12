//! Re-exported from [`scylla_auth::caller`]: the caller identity lives with
//! the access model, and every `crate::application::caller::...` path in this
//! crate keeps resolving through this module.

pub use scylla_auth::caller::*;
