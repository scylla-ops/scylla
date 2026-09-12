//! The access model, re-exported from [`scylla_auth::authz`] so that every
//! `crate::application::authz::...` path in this crate keeps resolving.
//!
//! [`GrantUseCases`] is the one part that is defined here rather than in
//! `scylla-auth`: revoking an App's grant also drops its live agent stream,
//! which ties it to this crate's dispatch ports.

pub use scylla_auth::authz::*;

pub mod grant;

pub use grant::GrantUseCases;
