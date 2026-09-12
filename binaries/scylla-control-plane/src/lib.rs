//! The Scylla control plane binary: the composition root that wires the
//! `scylla-db` adapters to the `scylla-core` use cases, and the runtime.
//!
//! The use cases, surfaces and configuration live in `scylla-core`; they are
//! re-exported below under their former paths so the composition root keeps
//! naming them as `crate::application::...`, `crate::config::...` and so on.

/// The domain model, re-exported from the [`scylla_domain`] kernel.
#[doc(no_inline)]
pub use scylla_domain::domain;

#[doc(no_inline)]
pub use scylla_core::{application, bootstrap, config, error, grpc, rest, tls};

pub mod infrastructure;

pub mod runtime;
pub mod startup;

pub use config::{BootstrapConfig, ControlPlaneConfig, CorsConfig, ServerConfig, UiConfig};
pub use error::{BootstrapError, ConfigError, StartupError};
pub use startup::{
    Services, SharedAuthUc, SharedGrantUc, SharedJobLogStreamUc, SharedJobLogUc, SharedJobUc,
    SharedOrgUc, SharedPipelineUc, SharedProjectUc, SharedUserUc, SharedWebhookIngressUc,
    build_cors_layer, init_services, run_server, shutdown_signal,
};
