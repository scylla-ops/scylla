//! Grant management. The grant types, the repository port and the rules live
//! in `scylla_auth::authz::grant`; the use case is here because revoking an
//! App's grant also drops its live agent stream through this crate's dispatch
//! ports.

mod use_case;

pub use use_case::GrantUseCases;
