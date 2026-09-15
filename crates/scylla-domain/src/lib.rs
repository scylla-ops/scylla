//! No sqlx, tonic, cedar, reqwest, lettre, oauth2 or argon2 here: the agent links this crate.

pub mod domain;

pub use domain::job::JobEvent;
