use crate::startup::{init_services, run_server, shutdown_signal};
use anyhow::{Context, Result};
use scylla_core::config::ControlPlaneConfig;
use scylla_extension::Extensions;
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;
use tracing::info;

/// Build the application services on `db` with the edition's `extensions`,
/// serve every surface on the configured listener until Ctrl+C or SIGTERM, then
/// close the pool.
///
/// Single composition root for the in-process control plane. Job dispatch and
/// log fan-out are in-process (the agent stream), so there is no broker or
/// recorder to boot; the web UI, the gRPC API and the webhook ingress share one
/// listener, so there is a single server to wait on.
///
/// The binary opens `db` (see `scylla_db::init_db`) rather than this function:
/// an edition may need the pool before the services exist, to run its own
/// migrations or to hand it to an extension.
pub async fn serve(config: ControlPlaneConfig, db: PgPool, extensions: Extensions) -> Result<()> {
    let token = CancellationToken::new();

    let services = init_services(&config, db.clone(), extensions)
        .await
        .context("init_services failed")?;

    // Ctrl+C / SIGTERM cancel the root token.
    let signal_token = token.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        signal_token.cancel();
    });

    // The server blocks until the token is cancelled.
    let server_token = token.clone();
    let result = run_server(&config, &services, async move {
        server_token.cancelled().await;
    })
    .await;

    token.cancel();

    info!("closing database pool");
    scylla_db::close_db(&db).await;

    result.context("run_server failed")?;
    Ok(())
}
