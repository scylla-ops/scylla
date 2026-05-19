#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("invalid broker URL `{url}`: {message}")]
    InvalidBrokerUrl { url: String, message: String },

    #[error("failed to connect to broker: {0}")]
    Connection(#[from] tonic::transport::Error),

    #[error("broker stream closed unexpectedly")]
    StreamClosed,

    #[error("failed to send to broker: {0}")]
    Send(#[from] tonic::Status),

    #[error("failed to deserialize message: {0}")]
    Deserialization(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("node {node_id} failed with exit code {exit_code}")]
    NodeFailed { node_id: String, exit_code: i32 },

    #[error("node {node_id} was killed by signal")]
    NodeKilled { node_id: String },

    #[error("node {node_id} cancelled by executor")]
    Cancelled { node_id: String },

    #[error("dangling dependencies — not all nodes could be scheduled (possible cycle)")]
    DanglingDeps,

    #[error("failed to spawn command: {0}")]
    Spawn(#[source] std::io::Error),

    #[error("failed to publish status: {0}")]
    Publish(String),

    #[error("node task panicked: {message}")]
    NodeTaskPanic { message: String },
}
