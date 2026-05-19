mod agent_listener;
mod config;
mod error;
mod log_listener;
mod status_listener;

use clap::Parser;
use config::RecorderConfig;
use scylla_core::application::{AgentUseCases, JobLogUseCases, JobUseCases};
use scylla_core::infrastructure::{PgAgentRepository, PgJobLogRepository, PgJobRepository};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = RecorderConfig::parse();
    info!(broker_url = %config.broker_url, db_url = %config.db_url, "starting scylla-recorder");

    // Connect to database
    let db = scylla_core::infrastructure::init_db(&config.database_config()).await?;
    info!("connected to database");

    let job_uc = Arc::new(JobUseCases::new(Arc::new(PgJobRepository::new(db.clone()))));
    let job_log_uc = Arc::new(JobLogUseCases::new(Arc::new(PgJobLogRepository::new(
        db.clone(),
    ))));
    let agent_uc = Arc::new(AgentUseCases::new(Arc::new(PgAgentRepository::new(db))));

    // Connect to broker
    let broker_channel = hermes_broker_client::connect(&config.broker_url, None).await?;
    info!("connected to broker");

    // Spawn listeners independently
    tokio::spawn(status_listener::run(broker_channel.clone(), job_uc));
    tokio::spawn(log_listener::run(broker_channel.clone(), job_log_uc));
    tokio::spawn(agent_listener::run_heartbeat(
        broker_channel.clone(),
        agent_uc.clone(),
    ));
    tokio::spawn(agent_listener::run_shutdown(broker_channel, agent_uc));

    info!("scylla-recorder running — press Ctrl+C to stop");

    tokio::signal::ctrl_c().await?;
    info!("scylla-recorder shut down");
    Ok(())
}
