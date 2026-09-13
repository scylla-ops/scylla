//! The server, configured with a builder: the one public way to run Scylla.
//!
//! Everything an edition binary contributes goes through [`Server`]'s methods,
//! and those methods are generic over what is contributed: an extension point
//! is registered by its trait, a gRPC service by its generated server type, a
//! whole capability as a [`Feature`]. Adding a feature to an edition therefore
//! never changes this crate. The Community binary uses the defaults.

use crate::feature::{Context, Feature};
use crate::startup::{init_services, run_server, shutdown_signal};
use crate::surface::Surface;
use anyhow::{Context as _, Result};
use scylla_core::config::ControlPlaneConfig;
use scylla_extension::Extensions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::info;

/// A control plane about to run: the configuration, the pool the binary
/// opened, and what the edition adds.
///
/// Built with [`Server::new`], refined with the fluent methods, started with
/// [`Server::serve`]. Every method has a default, so the Community binary is
/// `Server::new(config, db).serve()`.
pub struct Server {
    config: ControlPlaneConfig,
    db: PgPool,
    extensions: Extensions,
    features: Vec<Box<dyn Feature>>,
    surface: Surface,
}

impl Server {
    /// A server on `config` and the pool the binary opened (see
    /// `scylla_db::init_db`). The binary owns the pool so it can hand it to
    /// whatever else needs the database before anything starts.
    #[must_use]
    pub fn new(config: ControlPlaneConfig, db: PgPool) -> Self {
        Self {
            config,
            db,
            extensions: Extensions::new(),
            features: Vec::new(),
            surface: Surface::default(),
        }
    }

    /// Register the edition's implementation of an extension point, named by
    /// its trait: `.extension::<dyn QuotaPolicy>(Arc::new(MyQuota))`. A point
    /// left unregistered gets the core's default. Registering the same point
    /// twice keeps the last one.
    #[must_use]
    pub fn extension<T: ?Sized + Send + Sync + 'static>(mut self, implementation: Arc<T>) -> Self {
        self.extensions.insert(implementation);
        self
    }

    /// Add a whole capability: its extension implementations, its migrations
    /// and its services, in the order [`Feature`] describes.
    #[must_use]
    pub fn feature(mut self, feature: impl Feature) -> Self {
        self.features.push(Box::new(feature));
        self
    }

    /// A gRPC service on the listener, its own gate. See [`Surface::grpc_service`].
    #[must_use]
    pub fn grpc_service<S>(mut self, service: S) -> Self
    where
        S: tower::Service<http::Request<tonic::body::Body>, Error = std::convert::Infallible>
            + tonic::server::NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Response: axum::response::IntoResponse,
        S::Future: Send + 'static,
    {
        self.surface.grpc_service(service);
        self
    }

    /// A gRPC service behind the core's bearer-token interceptor. See
    /// [`Surface::authenticated_grpc_service`].
    #[must_use]
    pub fn authenticated_grpc_service<S>(mut self, service: S) -> Self
    where
        S: tower::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<tonic::body::Body>,
                Error = std::convert::Infallible,
            > + tonic::server::NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
    {
        self.surface.authenticated_grpc_service(service);
        self
    }

    /// A descriptor set for reflection. See [`Surface::file_descriptor_set`].
    #[must_use]
    pub fn file_descriptor_set(mut self, set: &'static [u8]) -> Self {
        self.surface.file_descriptor_set(set);
        self
    }

    /// Plain HTTP routes. See [`Surface::http_routes`].
    #[must_use]
    pub fn http_routes(mut self, router: axum::Router) -> Self {
        self.surface.http_routes(router);
        self
    }

    /// Run the features' three steps around the core's services, serve every
    /// surface on the configured listener until Ctrl+C or SIGTERM, then close
    /// the pool.
    ///
    /// Single composition root for the in-process control plane. Job dispatch
    /// and log fan-out are in-process (the agent stream), so there is no broker
    /// or recorder to boot; the web UI, the gRPC API and the webhook ingress
    /// share one listener, so there is a single server to wait on.
    pub async fn serve(self) -> Result<()> {
        let Self {
            config,
            db,
            mut extensions,
            features,
            mut surface,
        } = self;

        for feature in &features {
            feature.extensions(&mut extensions);
        }
        for feature in &features {
            feature
                .prepare(&db)
                .await
                .context("feature prepare failed")?;
        }

        let services = init_services(&config, db.clone(), extensions.clone())
            .await
            .context("init_services failed")?;

        let ctx = Context {
            db: db.clone(),
            extensions,
            permissions: services.permission_checker.clone(),
            policy_control: services.permission_checker.clone(),
            visibility: services.permission_checker.clone(),
        };
        for feature in features {
            feature.install(&ctx, &mut surface);
        }

        // Ctrl+C / SIGTERM cancel the root token.
        let token = CancellationToken::new();
        let signal_token = token.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            signal_token.cancel();
        });

        // The server blocks until the token is cancelled.
        let server_token = token.clone();
        let result = run_server(&config, &services, surface, async move {
            server_token.cancelled().await;
        })
        .await;

        token.cancel();

        info!("closing database pool");
        scylla_db::close_db(&db).await;

        result.context("run_server failed")?;
        Ok(())
    }
}
