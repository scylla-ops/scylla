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

    let services = crate::init_services(&config)
        .await
        .context("init_services failed")?;
    let db_pool = services.db.clone();

    // ── Ctrl+C / SIGTERM → cancel root token ───────────────────────────
    let signal_token = token.clone();
    tokio::spawn(async move {
        crate::shutdown_signal().await;
        signal_token.cancel();
    });

    // ── Webhook ingress (separate HTTP port, optional) ─────────────────
    // Runs concurrently with the gRPC server and shuts down on the same token.
    let webhook_handle = config.webhook.clone().map(|wh| {
        let ingress = services.webhook_ingress_uc.clone();
        let wh_token = token.clone();
        tokio::spawn(async move {
            crate::run_webhook(wh.address, ingress, async move {
                wh_token.cancelled().await;
            })
            .await
        })
    });

    // ── API gRPC server (blocks until token cancelled) ─────────────────
    let api_token = token.clone();
    let api_result = crate::run_grpc(&config, &services, async move {
        api_token.cancelled().await;
    })
    .await;

    token.cancel();

    if let Some(handle) = webhook_handle {
        match handle.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => info!(error = %e, "webhook server stopped with error"),
            Err(e) => info!(error = %e, "webhook server task join error"),
        }
    }

    info!("closing database pool");
    scylla_core::infrastructure::close_db(&db_pool).await;

    api_result.context("api run_grpc failed")?;
    Ok(())
}
