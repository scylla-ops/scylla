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

pub struct Server {
    config: ControlPlaneConfig,
    db: PgPool,
    extensions: Extensions,
    features: Vec<Box<dyn Feature>>,
    surface: Surface,
}

impl Server {
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

    #[must_use]
    pub fn extension<T: ?Sized + Send + Sync + 'static>(mut self, implementation: Arc<T>) -> Self {
        self.extensions.insert(implementation);
        self
    }

    #[must_use]
    pub fn feature(mut self, feature: impl Feature) -> Self {
        self.features.push(Box::new(feature));
        self
    }

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

    #[must_use]
    pub fn file_descriptor_set(mut self, set: &'static [u8]) -> Self {
        self.surface.file_descriptor_set(set);
        self
    }

    #[must_use]
    pub fn http_routes(mut self, router: axum::Router) -> Self {
        self.surface.http_routes(router);
        self
    }

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

        let token = CancellationToken::new();
        let signal_token = token.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            signal_token.cancel();
        });

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
