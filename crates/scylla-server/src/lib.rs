//! The composition root: wires the `scylla-db` adapters, the `scylla-auth`
//! Cedar engine and the edition's extensions into the `scylla-core` use cases,
//! builds the single-listener server and runs it until shutdown.
//!
//! This is the crate an edition binary calls into. It sits above every other
//! library crate because it is the only place allowed to name concrete
//! implementations side by side: `scylla-core` is generic over its ports,
//! `scylla-db` implements them, and neither may know about the other's
//! concrete types.
//!
//! An edition binary is a `main.rs` of a few lines: [`cli::Cli::parse_as`],
//! [`cli::init_tracing`], [`cli::Cli::load_config`], `scylla_db::init_db`,
//! then [`Server::new`] refined with what the edition contributes (usually as
//! [`Feature`]s) and [`Server::serve`]. The Community one is
//! `binaries/scylla-ce/src/main.rs`.

pub mod cli;
mod feature;
mod server;
mod startup;
mod surface;

pub use feature::{Context, Feature, PrepareFuture};
pub use server::Server;
pub use surface::Surface;
