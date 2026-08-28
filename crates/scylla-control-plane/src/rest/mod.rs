//! HTTP/REST transport layer — the counterpart to [`crate::grpc`] for plain
//! HTTP endpoints. Inbound webhook ingress lives here; any future REST routes
//! join as sibling modules under this folder.
//!
//! (Named `rest` rather than `http` on purpose: a crate-root `mod http` would
//! shadow the external `http` crate that the rest of the API imports.)

pub mod webhook;
