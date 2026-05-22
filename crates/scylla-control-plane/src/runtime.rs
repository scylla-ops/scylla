use crate::config::ControlPlaneConfig;
use anyhow::{Context, Result};
use tokio_util::sync::CancellationToken;
use tracing::info;

/// Build the shared application services and run the gRPC API until shutdown.
/// Single composition root for the in-process control plane. Job dispatch and
/// log fan-out are in-process (the agent stream), so there is no broker or
/// recorder to boot.
pub async fn run(config: ControlPlaneConfig) -> Result<()> {
    let token = CancellationToken::new();

    let services = scylla_api::init_services(&config.api)
        .await
        .context("init_services failed")?;
    let db_pool = services.db.clone();

    // ── Ctrl+C / SIGTERM → cancel root token ───────────────────────────
    let signal_token = token.clone();
    tokio::spawn(async move {
        scylla_api::shutdown_signal().await;
        signal_token.cancel();
    });

    // ── API gRPC server (blocks until token cancelled) ─────────────────
    let api_token = token.clone();
    let api_result = scylla_api::run_grpc(&config.api, &services, async move {
        api_token.cancelled().await;
    })
    .await;

    token.cancel();

    info!("closing database pool");
    scylla_core::infrastructure::close_db(&db_pool).await;

    api_result.context("api run_grpc failed")?;
    Ok(())
}
