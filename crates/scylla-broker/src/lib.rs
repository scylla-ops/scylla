use hermes_broker_core::router::{Router, RouterConfig};
use hermes_broker_proto::broker_server::BrokerServer;
use hermes_broker_server::grpc::BrokerService;
use std::future::Future;
use std::net::SocketAddr;
use thiserror::Error;
use tonic::transport::Server;
use tracing::info;

#[derive(Debug, Clone)]
pub struct BrokerConfig {
    pub addr: SocketAddr,
    pub channel_capacity: usize,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            // SAFETY: const string parses; only fails on programmer typo.
            addr: "0.0.0.0:50052"
                .parse()
                .expect("hardcoded default broker addr"),
            channel_capacity: 8192,
        }
    }
}

#[derive(Debug, Error)]
pub enum BrokerError {
    #[error("gRPC reflection: {0}")]
    Reflection(String),
    #[error("gRPC serve: {0}")]
    Serve(#[from] tonic::transport::Error),
}

/// Run the broker until `shutdown` resolves. Spawns the hermes router
/// internally and binds a `BrokerServer` gRPC service to `config.addr`.
pub async fn run<F>(config: BrokerConfig, shutdown: F) -> Result<(), BrokerError>
where
    F: Future<Output = ()> + Send + 'static,
{
    let router_cfg = RouterConfig::default();
    let (router, router_tx) = Router::new(router_cfg, config.channel_capacity);
    tokio::spawn(router.run());

    let service = BrokerService::new(router_tx);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(hermes_broker_proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .map_err(|e| BrokerError::Reflection(e.to_string()))?;

    info!(addr = %config.addr, "scylla-broker listening");

    Server::builder()
        .http2_keepalive_interval(Some(std::time::Duration::from_secs(10)))
        .http2_keepalive_timeout(Some(std::time::Duration::from_secs(5)))
        .add_service(reflection)
        .add_service(BrokerServer::new(service))
        .serve_with_shutdown(config.addr, shutdown)
        .await?;

    info!("scylla-broker shut down");
    Ok(())
}
