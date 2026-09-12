//! The composition root: wires the `scylla-db` adapters, the `scylla-auth`
//! Cedar engine and the edition's extensions into the `scylla-core` use cases,
//! builds the single-listener server and runs it until shutdown.
//!
//! This is the crate an edition binary calls into. It sits above every other
//! library crate because it is the only place allowed to name concrete
//! implementations side by side: `scylla-core` is generic over its ports,
//! `scylla-db` implements them, and neither may know about the other's
//! concrete types. The Community and Enterprise binaries are each a `main.rs`
//! that loads a configuration, builds an [`scylla_extension::Extensions`] and
//! calls [`runtime::run`].

pub mod runtime;
pub mod startup;

pub use startup::{
    Services, SharedAuthUc, SharedGrantUc, SharedJobLogStreamUc, SharedJobLogUc, SharedJobUc,
    SharedOrgUc, SharedPipelineUc, SharedProjectUc, SharedUserUc, SharedWebhookIngressUc,
    build_cors_layer, init_services, run_server, shutdown_signal,
};
