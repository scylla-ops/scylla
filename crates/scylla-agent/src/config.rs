use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "scylla-agent", about = "Scylla pipeline execution agent")]
pub struct AgentConfig {
    /// Hermes broker gRPC URL
    #[arg(long, default_value = "http://127.0.0.1:50052")]
    pub broker_url: String,

    /// Queue group name for load-balanced dispatch
    #[arg(long, default_value = "agents")]
    pub queue_group: String,

    /// Subject to subscribe to for job dispatch
    #[arg(long, default_value = "scylla.jobs.dispatch")]
    pub dispatch_subject: String,
}
