use clap::Parser;
use scylla_agent::{Agent, AgentConfig};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = AgentConfig::parse();
    info!(
        control_plane_url = %config.control_plane_url,
        app_id = %config.app_id,
        "starting scylla-agent"
    );

    let agent = Agent::new(config);

    tokio::select! {
        result = agent.run() => result.map_err(anyhow::Error::from),
        () = shutdown_signal() => {
            info!("shutdown signal received");
            Ok(())
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        // INVARIANT: SIGTERM handler installation cannot fail at startup on supported platforms.
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
