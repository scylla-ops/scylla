use hermes_broker_core::router::{Router, RouterConfig};
use hermes_broker_proto::broker_server::BrokerServer;
use hermes_broker_server::grpc::BrokerService;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let addr = "0.0.0.0:50052".parse()?;

    let config = RouterConfig::default();
    let (router, router_tx) = Router::new(config, 8192);
    tokio::spawn(router.run());

    let service = BrokerService::new(router_tx);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(hermes_broker_proto::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!(%addr, "scylla-broker listening");

    Server::builder()
        .http2_keepalive_interval(Some(std::time::Duration::from_secs(10)))
        .http2_keepalive_timeout(Some(std::time::Duration::from_secs(5)))
        .add_service(reflection)
        .add_service(BrokerServer::new(service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    info!("scylla-broker shut down");
    Ok(())
}

async fn shutdown_signal() {
    // INVARIANT: Ctrl+C handler installation cannot fail at startup on supported platforms.
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    info!("shutdown signal received");
}
