use crate::config::ControlPlaneConfig;
use anyhow::{Context, Result};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

/// Boot the broker, build shared application services, attach the recorder
/// listeners, and run the gRPC API until shutdown. Single composition root for
/// the in-process control plane.
pub async fn run(config: ControlPlaneConfig) -> Result<()> {
    let token = CancellationToken::new();

    // ── Broker ─────────────────────────────────────────────────────────
    let broker_cfg = scylla_broker::BrokerConfig {
        addr: config.broker_server.addr,
        channel_capacity: config.broker_server.channel_capacity,
    };
    let broker_token = token.clone();
    let broker_task = tokio::spawn(async move {
        scylla_broker::run(broker_cfg, async move {
            broker_token.cancelled().await;
        })
        .await
    });

    // Give the broker a moment to bind before downstream services connect.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // ── Application services ───────────────────────────────────────────
    let services = scylla_api::init_services(&config.api)
        .await
        .context("init_services failed")?;
    let db_pool = services.db.clone();

    // ── Recorder listeners ─────────────────────────────────────────────
    let broker_channel = hermes_broker_client::connect(&config.api.broker.url, None)
        .await
        .with_context(|| format!("recorder failed to connect to broker {}", config.api.broker.url))?;
    let recorder_services = scylla_recorder::RecorderServices {
        job_uc: services.job_uc.clone(),
        job_log_uc: services.job_log_uc.clone(),
        agent_uc: services.agent_uc.clone(),
    };
    let recorder_handles = scylla_recorder::spawn_listeners(broker_channel, recorder_services);

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

    // API has stopped — initiate teardown of the rest.
    token.cancel();

    info!("aborting recorder listeners");
    for h in &recorder_handles {
        h.abort();
    }
    for h in recorder_handles {
        let _ = h.await;
    }

    info!("waiting for broker task to exit");
    match broker_task.await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => warn!(error = %e, "broker exited with error"),
        Err(e) => warn!(error = %e, "broker task join error"),
    }

    info!("closing database pool");
    scylla_core::infrastructure::close_db(&db_pool).await;

    api_result.context("api run_grpc failed")?;
    Ok(())
}
