//! Scylla's shared kernel: the domain model, plus the handful of types that
//! travel between the control plane and the agents.
//!
//! This crate is deliberately dependency-light. It links no database driver, no
//! HTTP or gRPC stack, no crypto and no mail client, so an agent can depend on
//! it without dragging in the server's world. Anything that talks to an external
//! system is an adapter and belongs in `scylla-control-plane` instead.
//!
//! Concretely: nothing here may depend on `sqlx`, `tonic`, `cedar-policy`,
//! `reqwest`, `lettre`, `oauth2` or `argon2`. Reaching for one of those is the
//! signal that the code belongs on the other side of the boundary.
//!
//! See [`domain`] for where a new file goes.

pub mod domain;

/// The agent-to-control-plane event vocabulary, re-exported at the root because
/// it is the one type an agent reaches for without caring which subject owns it.
pub use domain::job::JobEvent;
