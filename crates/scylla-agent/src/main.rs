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
    info!(broker_url = %config.broker_url, "starting scylla-agent");

    let agent = Agent::connect(config).await?;
    agent.run().await?;

    Ok(())
}
