//! Scoped grants, re-exported from [`scylla_auth::authz::grant`], plus the
//! grant management use case that stays in this crate (see the parent module).

pub use scylla_auth::authz::grant::*;

mod use_case;

pub use use_case::*;
