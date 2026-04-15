use clap::Parser;
use scylla_agent::{Agent, AgentConfig, PresencePublisher};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = AgentConfig::parse();
    let agent_id = config.resolved_agent_id();
    let hostname = config.resolved_hostname();
    let heartbeat_interval_secs = config.heartbeat_interval_secs;
    let heartbeat_interval = Duration::from_secs(heartbeat_interval_secs);

    info!(
        broker_url = %config.broker_url,
        agent_id = %agent_id,
        hostname = %hostname,
        "starting scylla-agent"
    );

    let agent = Agent::connect(config).await?;

    let presence = Arc::new(PresencePublisher::new(
        agent.channel(),
        agent_id.clone(),
        hostname.clone(),
        heartbeat_interval_secs,
    ));

    presence.publish_heartbeat().await;
    presence.clone().spawn_heartbeat_ticker(heartbeat_interval);

    let run_result = tokio::select! {
        result = agent.run() => result.map_err(anyhow::Error::from),
        _ = shutdown_signal() => {
            info!("shutdown signal received");
            Ok(())
        }
    };

    presence.publish_shutdown().await;
    tokio::time::sleep(Duration::from_millis(200)).await;

    run_result
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
