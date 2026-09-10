use crate::config::ControlPlaneConfig;
use anyhow::{Context, Result};
use tokio_util::sync::CancellationToken;
use tracing::info;

/// Build the shared application services and serve until shutdown.
/// Single composition root for the in-process control plane. Job dispatch and
/// log fan-out are in-process (the agent stream), so there is no broker or
/// recorder to boot — and since the web UI, the gRPC API and the webhook
/// ingress now share one listener, there is a single server to wait on.
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

    // ── The server (blocks until the token is cancelled) ───────────────
    let server_token = token.clone();
    let result = crate::run_server(&config, &services, async move {
        server_token.cancelled().await;
    })
    .await;

    token.cancel();

    info!("closing database pool");
    crate::infrastructure::close_db(&db_pool).await;

    result.context("run_server failed")?;
    Ok(())
}
