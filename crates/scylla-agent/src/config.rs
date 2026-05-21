use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "scylla-agent", about = "Scylla pipeline execution agent")]
pub struct AgentConfig {
    /// Control-plane gRPC URL (the worker stream + token endpoint).
    #[arg(
        long,
        env = "SCYLLA_CONTROL_PLANE_URL",
        default_value = "http://127.0.0.1:50051"
    )]
    pub control_plane_url: String,

    /// App identity. The agent authenticates as this App and acts under its
    /// worker grant.
    #[arg(long, env = "SCYLLA_APP_ID")]
    pub app_id: String,

    /// App secret, exchanged for a bearer token at startup.
    #[arg(long, env = "SCYLLA_APP_SECRET")]
    pub app_secret: String,

    /// Buffer size of the in-process channel feeding the worker up-stream.
    /// Each node emits ~3 messages (NodeStarted, log lines, NodeCompleted), so a
    /// fast sequential chain can queue several hundred before the stream drains.
    /// Too small ⇒ the executor stalls on `send().await`; too big ⇒ memory bloat.
    #[arg(long, default_value_t = 8192, value_parser = clap::value_parser!(u64).range(1..))]
    pub publish_buffer_size: u64,
}
