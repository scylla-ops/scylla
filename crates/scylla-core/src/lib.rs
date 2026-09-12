//! The Scylla core: use cases and their ports, the in-memory adapters, the
//! gRPC and HTTP surfaces, and the server configuration.
//!
//! The domain model itself lives in the `scylla-domain` kernel, which the agents
//! also depend on; the access model in `scylla-auth`; the Postgres adapters in
//! `scylla-db`. Nothing here names a database: every use case and handler is
//! generic over the repository and authorization ports, and the composition
//! root that picks the implementations lives above this crate.
//!
//! Editions plug in through [`scylla_extension`]: the use cases that have an
//! extension point hold the trait object and never know which edition built it.

/// The domain model, re-exported from the [`scylla_domain`] kernel.
///
/// This is what lets every `crate::domain::...` path in this crate keep
/// resolving now that the model lives in its own crate. Extracting the kernel
/// would otherwise have meant rewriting 484 import sites for no behavioural
/// gain, and buried the actual change under the churn.
///
/// `no_inline` on purpose: without it rustdoc copies all 119 pages of the model
/// into this crate's documentation, and a reader would reasonably conclude the
/// domain belongs here. The link points at the kernel instead, which is the
/// whole message of the split.
#[doc(no_inline)]
pub use scylla_domain::domain;

pub mod application;
pub mod infrastructure;

pub mod bootstrap;
pub mod config;
pub mod error;
pub mod rest;
pub mod tls;

pub mod grpc;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_support;

pub use grpc::{
    AuthContext, AuthHandler, JobHandler, OrganizationHandler, PipelineHandler, ProjectHandler,
    UserHandler, auth_interceptor, domain_error_to_status, domain_to_proto_metadata, job_to_proto,
    middleware::extract_auth_context, organization_to_proto, pipeline_to_proto, project_to_proto,
    proto_to_domain_pagination, user_to_proto,
};

pub use config::{BootstrapConfig, ControlPlaneConfig, CorsConfig, ServerConfig, UiConfig};
pub use error::{BootstrapError, ConfigError, StartupError};
