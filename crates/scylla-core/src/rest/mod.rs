//! HTTP/REST transport layer: the counterpart to [`crate::grpc`] for plain
//! HTTP endpoints. The liveness probe, the inbound webhook ingress and the web
//! UI live here; any future REST routes join as sibling modules under this
//! folder.
//!
//! (Named `rest` rather than `http` on purpose: a crate-root `mod http` would
//! shadow the external `http` crate that the rest of the API imports.)

pub mod health;
pub mod ui;
pub mod webhook;
